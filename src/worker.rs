// src/worker.rs

use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use tracing::{error, info, warn};

use crate::tweaks::{Tweak, TweakId, TweakStatus};

/// Define worker messages and results
#[derive(Clone)]
pub enum WorkerTask {
    ReadInitialState {
        id: TweakId,
        tweak: Arc<Mutex<Tweak>>,
    },
    ApplyTweak {
        id: TweakId,
        tweak: Arc<Mutex<Tweak>>,
    },
    RevertTweak {
        id: TweakId,
        tweak: Arc<Mutex<Tweak>>,
    },
    Shutdown,
}

#[derive(Clone)]
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
    task_sender: mpsc::Sender<WorkerTask>,
    result_receiver: mpsc::Receiver<WorkerResult>,
}

impl WorkerPool {
    /// Creates a new `WorkerPool` with the specified number of workers.
    pub fn new(num_workers: usize) -> Self {
        let (task_sender, task_receiver) = mpsc::channel::<WorkerTask>();
        let task_receiver = Arc::new(Mutex::new(task_receiver));

        let (result_sender, result_receiver) = mpsc::channel::<WorkerResult>();

        for _ in 0..num_workers {
            let task_receiver = Arc::clone(&task_receiver);
            let result_sender = result_sender.clone();

            thread::spawn(move || {
                worker_loop(task_receiver, result_sender);
            });
        }

        WorkerPool {
            task_sender,
            result_receiver,
        }
    }

    /// Sends a task to the worker pool.
    pub fn send_task(&self, task: WorkerTask) -> Result<(), mpsc::SendError<WorkerTask>> {
        self.task_sender.send(task)
    }

    /// Attempts to receive a result from the worker pool without blocking.
    pub fn try_recv_result(&self) -> Option<WorkerResult> {
        self.result_receiver.try_recv().ok()
    }

    /// Sends shutdown signals to all workers.
    pub fn shutdown(&self, num_workers: usize) {
        for _ in 0..num_workers {
            if let Err(e) = self.task_sender.send(WorkerTask::Shutdown) {
                error!("Failed to send Shutdown task: {}", e);
            }
        }
    }
}

/// The main loop for each worker thread.
fn worker_loop(
    task_receiver: Arc<Mutex<Receiver<WorkerTask>>>,
    result_sender: Sender<WorkerResult>,
) {
    loop {
        let task = {
            let receiver = task_receiver.lock().unwrap();
            receiver.recv()
        };

        match task {
            Ok(task) => match task {
                WorkerTask::ReadInitialState { id, tweak } => {
                    handle_read_initial_state(id, tweak, &result_sender);
                }
                WorkerTask::ApplyTweak { id, tweak } => {
                    handle_apply_tweak(id, tweak, &result_sender);
                }
                WorkerTask::RevertTweak { id, tweak } => {
                    handle_revert_tweak(id, tweak, &result_sender);
                }
                WorkerTask::Shutdown => {
                    info!("Worker: received Shutdown task.");
                    if let Err(e) = result_sender.send(WorkerResult::ShutdownComplete) {
                        error!("Worker: failed to send ShutdownComplete: {e}");
                    }
                    break;
                }
            },
            Err(_) => {
                // Channel closed, terminate the worker
                info!("Worker: detected closed task channel. Terminating.",);
                break;
            }
        }
    }
    info!("Worker shut down.");
}

/// Handles the ReadInitialState task.
fn handle_read_initial_state(
    tweak_id: TweakId,
    tweak: Arc<Mutex<Tweak>>,
    result_sender: &Sender<WorkerResult>,
) {
    let tweak_clone = {
        let tweak_guard = tweak.lock().unwrap();
        tweak_guard.clone()
    };

    info!("Worker: Reading initial state for {tweak_id:?}");

    // Perform the blocking read operation **without holding any locks**
    let is_enabled = match tweak_clone.initial_state(tweak_id) {
        Ok(state) => state,
        Err(e) => {
            warn!("Worker: Failed to read initial state for {tweak_id:?}: {e:?}");

            // Update the tweak status to Failed
            {
                let tweak_guard = tweak.lock().unwrap();
                tweak_guard.set_status(TweakStatus::Failed(e.to_string()));
            }

            // Send failure result
            if let Err(send_err) = result_sender.send(WorkerResult::InitialStateRead {
                id: tweak_id,
                success: false,
                error: Some(e.to_string()),
            }) {
                error!(
                    "Worker: failed to send InitialStateRead failure for {tweak_id:?}: {send_err}"
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

    info!("Worker:: Initial state read successfully for {tweak_id:?}. Enabled: {is_enabled}");

    // Send success result
    if let Err(e) = result_sender.send(WorkerResult::InitialStateRead {
        id: tweak_id,
        success: true,
        error: None,
    }) {
        error!("Worker: failed to send InitialStateRead result for {tweak_id:?}: {e}");
    }
}

/// Handles the ApplyTweak task.
fn handle_apply_tweak(
    tweak_id: TweakId,
    tweak: Arc<Mutex<Tweak>>,
    result_sender: &Sender<WorkerResult>,
) {
    info!("Worker: Applying tweak {tweak_id:?}");

    // Perform the blocking apply operation **without holding any locks**
    let apply_result = {
        let tweak_guard = tweak.lock().unwrap();
        tweak_guard.set_status(TweakStatus::Applying);
        tweak_guard.apply(tweak_id)
    };

    // Step 3: Update status and other fields based on the result
    match apply_result {
        Ok(_) => {
            {
                let tweak_guard = tweak.lock().unwrap();
                tweak_guard.set_enabled();
                tweak_guard.set_status(TweakStatus::Idle);

                if tweak_guard.requires_reboot {
                    tweak_guard.pending_reboot();
                    info!("Worker: {tweak_id:?} applied successfully. Pending reboot.",);
                } else {
                    info!("Worker: {tweak_id:?} applied successfully. No reboot required.");
                }
            }

            // Send success result
            if let Err(e) = result_sender.send(WorkerResult::TweakApplied {
                id: tweak_id,
                success: true,
                error: None,
            }) {
                error!("Worker: failed to send TweakApplied result for {tweak_id:?}: {e}");
            }
        }
        Err(e) => {
            {
                let tweak_guard = tweak.lock().unwrap();
                tweak_guard.set_status(TweakStatus::Failed(e.to_string()));
            }

            warn!(
                "Worker: Failed to apply tweak {tweak_id:?}: {:?}",
                tweak.lock().unwrap().get_status()
            );

            // Send failure result
            if let Err(send_err) = result_sender.send(WorkerResult::TweakApplied {
                id: tweak_id,
                success: false,
                error: Some(e.to_string()),
            }) {
                error!("Worker: failed to send TweakApplied failure for {tweak_id:?}: {send_err}");
            }
        }
    }
}

/// Handles the RevertTweak task.
fn handle_revert_tweak(
    tweak_id: TweakId,
    tweak: Arc<Mutex<Tweak>>,
    result_sender: &Sender<WorkerResult>,
) {
    info!("Worker: Reverting tweak {tweak_id:?}");

    let revert_result = {
        let tweak_guard = tweak.lock().unwrap();
        tweak_guard.set_status(TweakStatus::Applying);
        tweak_guard.revert(tweak_id)
    };

    // Step 3: Update status and other fields based on the result
    match revert_result {
        Ok(_) => {
            {
                let tweak_guard = tweak.lock().unwrap();
                tweak_guard.set_disabled();
                tweak_guard.set_status(TweakStatus::Idle);

                if tweak_guard.requires_reboot {
                    tweak_guard.pending_reboot();
                }
            }

            info!("Worker: {tweak_id:?} reverted successfully.");

            // Send success result
            if let Err(e) = result_sender.send(WorkerResult::TweakReverted {
                id: tweak_id,
                success: true,
                error: None,
            }) {
                error!("Worker: failed to send TweakReverted result for {tweak_id:?}: {e}");
            }
        }
        Err(e) => {
            {
                let tweak_guard = tweak.lock().unwrap();
                tweak_guard.set_status(TweakStatus::Failed(e.to_string()));
            }

            warn!(
                "Worker: Failed to revert tweak {tweak_id:?}: {:?}",
                tweak.lock().unwrap().get_status()
            );

            // Send failure result
            if let Err(send_err) = result_sender.send(WorkerResult::TweakReverted {
                id: tweak_id,
                success: false,
                error: Some(e.to_string()),
            }) {
                error!("Worker: failed to send TweakReverted failure for {tweak_id:?}: {send_err}");
            }
        }
    }
}
