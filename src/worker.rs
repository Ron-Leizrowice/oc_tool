// src/worker.rs

use std::{
    panic,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel::{unbounded, Receiver, Sender};

use crate::{
    actions::{Tweak, TweakAction},
    tweaks::TweakId,
};

#[derive(Clone, Debug)]
pub enum WorkerMessage {
    ExecuteTweak {
        tweak: Arc<Mutex<Tweak>>,
        is_toggle: bool,
    },
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum WorkerResult {
    TweakCompleted {
        id: TweakId,
        success: bool,
        error: Option<String>,
    },
}

pub struct TweakExecutor {
    pub sender: Sender<WorkerMessage>,
    pub receiver: Receiver<WorkerResult>,
}

impl TweakExecutor {
    pub fn new() -> Self {
        let (task_sender, task_receiver) = unbounded::<WorkerMessage>();
        let (result_sender, result_receiver) = unbounded::<WorkerResult>();

        thread::spawn(move || {
            // Wrap the entire thread in a catch_unwind to prevent panics from terminating the thread
            let thread_result = panic::catch_unwind(|| {
                while let Ok(message) = task_receiver.recv() {
                    match message {
                        WorkerMessage::ExecuteTweak { tweak, is_toggle } => {
                            // Attempt to lock the tweak
                            let tweak_guard = match tweak.lock() {
                                Ok(guard) => guard,
                                Err(poisoned) => {
                                    tracing::error!("Failed to lock tweak: {:?}", poisoned);
                                    // Attempt to extract the TweakId even if the lock is poisoned
                                    let tweak_id = poisoned.into_inner().id;
                                    let _ = result_sender.send(WorkerResult::TweakCompleted {
                                        id: tweak_id,
                                        success: false,
                                        error: Some("Failed to acquire lock on tweak.".to_string()),
                                    });
                                    continue;
                                }
                            };

                            let tweak_id = tweak_guard.id;
                            tracing::info!(
                                "Starting tweak {:?} as {:?}",
                                tweak_id,
                                if is_toggle { "Toggle" } else { "Apply" }
                            );

                            // Clone necessary data to avoid holding the lock during execution
                            let tweak_clone = tweak_guard.clone();
                            drop(tweak_guard); // Explicitly drop the lock

                            // Execute the tweak outside the lock
                            let execution_result = if is_toggle {
                                match tweak_clone.is_enabled() {
                                    Ok(enabled) => {
                                        if enabled {
                                            tweak_clone.revert()
                                        } else {
                                            tweak_clone.apply()
                                        }
                                    }
                                    Err(e) => Err(e),
                                }
                            } else {
                                tweak_clone.apply()
                            };

                            match execution_result {
                                Ok(_) => {
                                    tracing::info!("Tweak {:?} completed successfully.", tweak_id);
                                    let _ = result_sender.send(WorkerResult::TweakCompleted {
                                        id: tweak_id,
                                        success: true,
                                        error: None,
                                    });
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to execute tweak {:?}: {:?}",
                                        tweak_id,
                                        e
                                    );
                                    let _ = result_sender.send(WorkerResult::TweakCompleted {
                                        id: tweak_id,
                                        success: false,
                                        error: Some(e.to_string()),
                                    });
                                }
                            }
                        }
                        WorkerMessage::Shutdown => {
                            tracing::info!("TweakExecutor received shutdown signal.");
                            break;
                        }
                    }
                }
            });

            if let Err(e) = thread_result {
                tracing::error!("Worker thread panicked: {:?}", e);
                // Optionally, you can attempt to notify the UI about the panic
            }

            tracing::info!("TweakExecutor thread terminating.");
        });

        Self {
            sender: task_sender,
            receiver: result_receiver,
        }
    }
}
