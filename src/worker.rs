// src/worker.rs

use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

use dashmap::DashMap;
use tracing::{error, info, warn};

use crate::tweaks::{Tweak, TweakId, TweakStatus};

/// Define worker messages and results
#[derive(Clone)]
pub enum WorkerTask {
    ReadInitialState { id: TweakId },
    ApplyTweak { id: TweakId },
    RevertTweak { id: TweakId },
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
    task_sender: Sender<WorkerTask>,
    result_receiver: Receiver<WorkerResult>,
}

impl WorkerPool {
    /// Creates a new `WorkerPool` and starts its event loop in a separate thread.
    pub fn new(num_workers: usize, tweaks: Arc<DashMap<TweakId, Tweak>>) -> Self {
        let (task_sender, task_receiver) = mpsc::channel::<WorkerTask>();
        let (result_sender, result_receiver) = mpsc::channel::<WorkerResult>();

        // Clone tweaks for use in the worker pool thread
        let tweaks_clone = Arc::clone(&tweaks);

        // Start the WorkerPool event loop in a new thread
        thread::spawn(move || {
            Self::run_worker_pool(num_workers, task_receiver, result_sender, tweaks_clone);
        });

        WorkerPool {
            task_sender,
            result_receiver,
        }
    }

    /// The event loop that runs in the WorkerPool thread.
    fn run_worker_pool(
        num_workers: usize,
        task_receiver: Receiver<WorkerTask>,
        result_sender: Sender<WorkerResult>,
        tweaks: Arc<DashMap<TweakId, Tweak>>,
    ) {
        // Create channels for communicating with worker threads
        let mut worker_task_senders = Vec::new();
        let mut worker_result_receivers = Vec::new();

        for worker_id in 0..num_workers {
            let (worker_task_sender, worker_task_receiver) = mpsc::channel::<WorkerTask>();
            let (worker_result_sender, worker_result_receiver) = mpsc::channel::<WorkerResult>();

            let tweaks_clone = Arc::clone(&tweaks);

            // Start worker threads
            thread::spawn(move || {
                worker_loop(
                    worker_id,
                    worker_task_receiver,
                    worker_result_sender,
                    tweaks_clone,
                );
            });

            worker_task_senders.push(worker_task_sender);
            worker_result_receivers.push(worker_result_receiver);
        }

        let mut next_worker = 0;

        loop {
            // Receive a task from the UI thread
            match task_receiver.recv() {
                Ok(task) => {
                    // Check for shutdown
                    if let WorkerTask::Shutdown = task {
                        // Send shutdown to all workers
                        for sender in &worker_task_senders {
                            let _ = sender.send(WorkerTask::Shutdown);
                        }
                        break;
                    }

                    // Send the task to the next worker in a round-robin fashion
                    if let Some(worker_sender) = worker_task_senders.get(next_worker) {
                        if let Err(e) = worker_sender.send(task.clone()) {
                            error!("Failed to send task to worker {}: {}", next_worker, e);
                        }
                        next_worker = (next_worker + 1) % num_workers;
                    }
                }
                Err(e) => {
                    error!("WorkerPool: task receiver disconnected: {}", e);
                    break;
                }
            }

            // Collect results from workers and send them to the UI thread
            for receiver in &worker_result_receivers {
                while let Ok(result) = receiver.try_recv() {
                    if let Err(e) = result_sender.send(result) {
                        error!("WorkerPool: failed to send result to UI thread: {}", e);
                    }
                }
            }
        }

        // Collect any remaining results from workers
        for receiver in &worker_result_receivers {
            while let Ok(result) = receiver.recv() {
                if let Err(e) = result_sender.send(result) {
                    error!("WorkerPool: failed to send result to UI thread: {}", e);
                }
            }
        }

        info!("WorkerPool has shut down gracefully.");
    }

    /// Sends a task to the worker pool.
    pub fn send_task(&self, task: WorkerTask) -> Result<(), mpsc::SendError<WorkerTask>> {
        self.task_sender.send(task)
    }

    /// Attempts to receive a result from the worker pool without blocking.
    pub fn try_recv_result(&self) -> Option<WorkerResult> {
        self.result_receiver.try_recv().ok()
    }

    /// Sends a shutdown signal to the worker pool.
    pub fn shutdown(&self) {
        if let Err(e) = self.task_sender.send(WorkerTask::Shutdown) {
            error!("Failed to send Shutdown task to WorkerPool: {}", e);
        }
    }
}

/// The main loop for each worker thread.
fn worker_loop(
    worker_id: usize,
    task_receiver: Receiver<WorkerTask>,
    result_sender: Sender<WorkerResult>,
    tweaks: Arc<DashMap<TweakId, Tweak>>,
) {
    info!("Worker {} started.", worker_id);
    loop {
        match task_receiver.recv() {
            Ok(task) => match task {
                WorkerTask::ReadInitialState { id } => {
                    handle_read_initial_state(id, &tweaks, &result_sender);
                }
                WorkerTask::ApplyTweak { id } => {
                    handle_apply_tweak(id, &tweaks, &result_sender);
                }
                WorkerTask::RevertTweak { id } => {
                    handle_revert_tweak(id, &tweaks, &result_sender);
                }
                WorkerTask::Shutdown => {
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
    tweak_id: TweakId,
    tweaks: &DashMap<TweakId, Tweak>,
    result_sender: &Sender<WorkerResult>,
) {
    info!("Reading initial state for {:?}.", tweak_id);

    // Step 1: Acquire the lock briefly to clone the tweak
    let tweak_clone = if let Some(tweak) = tweaks.get(&tweak_id) {
        tweak.clone()
    } else {
        // Handle tweak not found
        warn!("Tweak {:?} not found during ReadInitialState.", tweak_id);
        let _ = result_sender.send(WorkerResult::InitialStateRead {
            id: tweak_id,
            success: false,
            error: Some("Tweak not found".to_string()),
        });
        return;
    };

    // Step 2: Perform the initial state read without holding the lock
    let initial_state_result = tweak_clone.initial_state(tweak_id);

    // Step 3: Reacquire the lock to update the tweak
    match initial_state_result {
        Ok(is_enabled) => {
            if let Some(mut tweak) = tweaks.get_mut(&tweak_id) {
                tweak.set_enabled(is_enabled);
                tweak.set_status(TweakStatus::Idle);
                if tweak.requires_reboot {
                    tweak.set_pending_reboot(is_enabled);
                }
            }
            info!(
                "Initial state read successfully for {:?}. Enabled: {}",
                tweak_id, is_enabled
            );
            let _ = result_sender.send(WorkerResult::InitialStateRead {
                id: tweak_id,
                success: true,
                error: None,
            });
        }
        Err(e) => {
            if let Some(mut tweak) = tweaks.get_mut(&tweak_id) {
                tweak.set_status(TweakStatus::Failed(e.to_string()));
            }
            warn!("Failed to read initial state for {:?}: {:?}", tweak_id, e);
            let _ = result_sender.send(WorkerResult::InitialStateRead {
                id: tweak_id,
                success: false,
                error: Some(e.to_string()),
            });
        }
    }
}

/// Handles the ApplyTweak task.
fn handle_apply_tweak(
    tweak_id: TweakId,
    tweaks: &DashMap<TweakId, Tweak>,
    result_sender: &Sender<WorkerResult>,
) {
    info!("Applying tweak {:?}.", tweak_id);

    // Clone the tweak without holding the lock
    let tweak_clone = if let Some(tweak) = tweaks.get(&tweak_id) {
        tweak.clone()
    } else {
        // Handle tweak not found
        warn!("Tweak {:?} not found during ApplyTweak.", tweak_id);
        let _ = result_sender.send(WorkerResult::TweakApplied {
            id: tweak_id,
            success: false,
            error: Some("Tweak not found".to_string()),
        });
        return;
    };

    // Perform the apply operation
    let apply_result = tweak_clone.apply(tweak_id);

    // Send the result back to the main thread
    match apply_result {
        Ok(_) => {
            let _ = result_sender.send(WorkerResult::TweakApplied {
                id: tweak_id,
                success: true,
                error: None,
            });
        }
        Err(e) => {
            let _ = result_sender.send(WorkerResult::TweakApplied {
                id: tweak_id,
                success: false,
                error: Some(e.to_string()),
            });
        }
    }
}
/// Handles the RevertTweak task.
fn handle_revert_tweak(
    tweak_id: TweakId,
    tweaks: &DashMap<TweakId, Tweak>,
    result_sender: &Sender<WorkerResult>,
) {
    info!("Reverting tweak {:?}.", tweak_id);

    // Step 1: Acquire the lock briefly to clone the tweak
    let tweak_clone = if let Some(tweak) = tweaks.get(&tweak_id) {
        tweak.clone()
    } else {
        // Handle tweak not found
        warn!("Tweak {:?} not found during RevertTweak.", tweak_id);
        let _ = result_sender.send(WorkerResult::TweakReverted {
            id: tweak_id,
            success: false,
            error: Some("Tweak not found".to_string()),
        });
        return;
    };

    // Step 2: Perform the revert operation without holding the lock
    let revert_result = tweak_clone.revert(tweak_id);

    // Step 3: Reacquire the lock to update the tweak
    match revert_result {
        Ok(_) => {
            if let Some(mut tweak) = tweaks.get_mut(&tweak_id) {
                tweak.set_enabled(false);
                tweak.set_status(TweakStatus::Idle);
                if tweak.requires_reboot {
                    tweak.set_pending_reboot(false);
                }
            }
            info!("Tweak {:?} reverted successfully.", tweak_id);
            let _ = result_sender.send(WorkerResult::TweakReverted {
                id: tweak_id,
                success: true,
                error: None,
            });
        }
        Err(e) => {
            if let Some(mut tweak) = tweaks.get_mut(&tweak_id) {
                tweak.set_status(TweakStatus::Failed(e.to_string()));
            }
            warn!("Failed to revert tweak {:?}: {:?}", tweak_id, e);
            let _ = result_sender.send(WorkerResult::TweakReverted {
                id: tweak_id,
                success: false,
                error: Some(e.to_string()),
            });
        }
    }
}
