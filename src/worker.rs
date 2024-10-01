// src/worker.rs

use std::{
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel::{unbounded, Receiver, Sender};
use tracing::{error, info, warn};

use crate::tweaks::{Tweak, TweakId, TweakStatus};

/// Define worker messages and results
#[derive(Clone)]
pub enum Task {
    ReadInitialState { tweak: Arc<Mutex<Tweak>> },
    ApplyTweak { tweak: Arc<Mutex<Tweak>> },
    RevertTweak { tweak: Arc<Mutex<Tweak>> },
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum WorkerResult {
    TweakApplied {
        id: TweakId,
        success: bool,
        error: Option<String>,
    },
    TweakReverted {
        id: TweakId,
        success: bool,
        error: Option<String>,
    },
    InitialStateRead {
        id: TweakId,
        success: bool,
        error: Option<String>,
    },
    ShutdownComplete,
}

/// Manages a pool of worker threads that process tasks concurrently.
pub struct WorkerPool {
    task_sender: Sender<Task>,
    result_receiver: Receiver<WorkerResult>,
}

impl WorkerPool {
    /// Creates a new `WorkerPool` with the specified number of workers.
    pub fn new(num_workers: usize) -> Self {
        let (task_sender, task_receiver) = unbounded::<Task>();
        let (result_sender, result_receiver) = unbounded::<WorkerResult>();

        for worker_id in 0..num_workers {
            let task_receiver = task_receiver.clone();
            let result_sender = result_sender.clone();

            thread::spawn(move || {
                worker_loop(worker_id, task_receiver, result_sender);
            });
        }

        WorkerPool {
            task_sender,
            result_receiver,
        }
    }

    /// Sends a task to the worker pool.
    pub fn send_task(&self, task: Task) -> Result<(), crossbeam::channel::SendError<Task>> {
        self.task_sender.send(task)
    }

    /// Attempts to receive a result from the worker pool without blocking.
    pub fn try_recv_result(&self) -> Option<WorkerResult> {
        self.result_receiver.try_recv().ok()
    }

    /// Sends shutdown signals to all workers.
    pub fn shutdown(&self, num_workers: usize) {
        for _ in 0..num_workers {
            if let Err(e) = self.task_sender.send(Task::Shutdown) {
                error!("Failed to send Shutdown task: {}", e);
            }
        }
    }
}

/// The main loop for each worker thread.
fn worker_loop(
    worker_id: usize,
    task_receiver: Receiver<Task>,
    result_sender: Sender<WorkerResult>,
) {
    info!("Worker {} started.", worker_id);
    loop {
        match task_receiver.recv() {
            Ok(task) => match task {
                Task::ReadInitialState { tweak } => {
                    handle_read_initial_state(worker_id, tweak, &result_sender);
                }
                Task::ApplyTweak { tweak } => {
                    handle_apply_tweak(worker_id, tweak, &result_sender);
                }
                Task::RevertTweak { tweak } => {
                    handle_revert_tweak(worker_id, tweak, &result_sender);
                }
                Task::Shutdown => {
                    info!("Worker {} received Shutdown task.", worker_id);
                    if let Err(e) = result_sender.send(WorkerResult::ShutdownComplete) {
                        error!(
                            "Worker {} failed to send ShutdownComplete: {}",
                            worker_id, e
                        );
                    }
                    break;
                }
            },
            Err(_) => {
                // Channel closed, terminate the worker
                info!(
                    "Worker {} detected closed task channel. Terminating.",
                    worker_id
                );
                break;
            }
        }
    }
    info!("Worker {} has shut down.", worker_id);
}

/// Handles the ReadInitialState task.
fn handle_read_initial_state(
    worker_id: usize,
    tweak: Arc<Mutex<Tweak>>,
    result_sender: &Sender<WorkerResult>,
) {
    let tweak_clone = {
        let tweak_guard = tweak.lock().unwrap();
        tweak_guard.clone()
    };

    info!(
        "Worker {}: Reading initial state for {:?}",
        worker_id, tweak_clone.id
    );

    // Perform the blocking read operation **without holding any locks**
    let is_enabled = match tweak_clone.initial_state() {
        Ok(state) => state,
        Err(e) => {
            warn!(
                "Worker {}: Failed to read initial state for {:?}: {:?}",
                worker_id, tweak_clone.id, e
            );

            // Update the tweak status to Failed
            {
                let tweak_guard = tweak.lock().unwrap().clone();
                tweak_guard.set_status(TweakStatus::Failed(e.to_string()));
            }

            // Send failure result
            if let Err(send_err) = result_sender.send(WorkerResult::InitialStateRead {
                id: tweak_clone.id,
                success: false,
                error: Some(e.to_string()),
            }) {
                error!(
                    "Worker {} failed to send InitialStateRead failure for {:?}: {}",
                    worker_id, tweak_clone.id, send_err
                );
            }
            return;
        }
    };

    // Update the tweak's enabled state and status to Idle
    {
        let tweak_guard = tweak.lock().unwrap();
        match is_enabled {
            true => tweak_guard.set_enabled(),
            false => tweak_guard.set_disabled(),
        }
        tweak_guard.set_status(TweakStatus::Idle);
    }

    info!(
        "Worker {}: Initial state read successfully for {:?}. Enabled: {}",
        worker_id, tweak_clone.id, is_enabled
    );

    // Send success result
    if let Err(e) = result_sender.send(WorkerResult::InitialStateRead {
        id: tweak_clone.id,
        success: true,
        error: None,
    }) {
        error!(
            "Worker {} failed to send InitialStateRead result for {:?}: {}",
            worker_id, tweak_clone.id, e
        );
    }
}

/// Handles the ApplyTweak task.
fn handle_apply_tweak(
    worker_id: usize,
    tweak: Arc<Mutex<Tweak>>,
    result_sender: &Sender<WorkerResult>,
) {
    let tweak_id;
    {
        let tweak_guard = tweak.lock().unwrap();
        tweak_id = tweak_guard.id;
    }

    info!("Worker {}: Applying tweak {:?}", worker_id, tweak_id);

    // Perform the blocking apply operation **without holding any locks**
    let apply_result = {
        let tweak_guard = tweak.lock().unwrap();
        tweak_guard.set_status(TweakStatus::Applying);
        tweak_guard.apply()
    };

    // Step 3: Update status and other fields based on the result
    match apply_result {
        Ok(_) => {
            let tweak_guard = tweak.lock().unwrap();
            tweak_guard.set_enabled();
            tweak_guard.set_status(TweakStatus::Idle);

            if tweak_guard.requires_reboot {
                tweak_guard.pending_reboot();
                info!(
                    "Worker {}: {:?} applied successfully. Pending reboot.",
                    worker_id, tweak_id
                );
            } else {
                info!(
                    "Worker {}: {:?} applied successfully. No reboot required.",
                    worker_id, tweak_id
                );
            }

            // Send success result
            if let Err(e) = result_sender.send(WorkerResult::TweakApplied {
                id: tweak_id,
                success: true,
                error: None,
            }) {
                error!(
                    "Worker {} failed to send TweakApplied result for {:?}: {}",
                    worker_id, tweak_id, e
                );
            }
        }
        Err(e) => {
            {
                let tweak_guard = tweak.lock().unwrap();
                tweak_guard.set_status(TweakStatus::Failed(e.to_string()));
            }

            warn!(
                "Worker {}: Failed to apply tweak {:?}: {:?}",
                worker_id,
                tweak_id,
                tweak.lock().unwrap().get_status()
            );

            // Send failure result
            if let Err(send_err) = result_sender.send(WorkerResult::TweakApplied {
                id: tweak_id,
                success: false,
                error: Some(e.to_string()),
            }) {
                error!(
                    "Worker {} failed to send TweakApplied failure for {:?}: {}",
                    worker_id, tweak_id, send_err
                );
            }
        }
    }
}

/// Handles the RevertTweak task.
fn handle_revert_tweak(
    worker_id: usize,
    tweak: Arc<Mutex<Tweak>>,
    result_sender: &Sender<WorkerResult>,
) {
    let tweak_id;
    {
        let tweak_guard = tweak.lock().unwrap();
        tweak_id = tweak_guard.id;
    }

    info!("Worker {}: Reverting tweak {:?}", worker_id, tweak_id);

    let revert_result = {
        let tweak_guard = tweak.lock().unwrap();
        tweak_guard.set_status(TweakStatus::Applying);
        tweak_guard.revert()
    };

    // Step 3: Update status and other fields based on the result
    match revert_result {
        Ok(_) => {
            let tweak_guard = tweak.lock().unwrap();
            tweak_guard.set_disabled();
            tweak_guard.set_status(TweakStatus::Idle);

            if tweak_guard.requires_reboot {
                tweak_guard.pending_reboot();

                // Send success result
                if let Err(e) = result_sender.send(WorkerResult::TweakReverted {
                    id: tweak_id,
                    success: true,
                    error: None,
                }) {
                    error!(
                        "Worker {} failed to send TweakReverted result for {:?}: {}",
                        worker_id, tweak_id, e
                    );
                }
            }
        }
        Err(e) => {
            let tweak_guard = tweak.lock().unwrap();
            tweak_guard.set_status(TweakStatus::Failed(e.to_string()));

            warn!(
                "Worker {}: Failed to revert tweak {:?}: {:?}",
                worker_id,
                tweak_id,
                tweak.lock().unwrap().get_status()
            );

            // Send failure result
            if let Err(send_err) = result_sender.send(WorkerResult::TweakReverted {
                id: tweak_id,
                success: false,
                error: Some(e.to_string()),
            }) {
                error!(
                    "Worker {} failed to send TweakReverted failure for {:?}: {}",
                    worker_id, tweak_id, send_err
                );
            }
        }
    }
}
