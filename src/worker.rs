// src/worker.rs

use std::{
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel::{unbounded, Receiver, Sender};

use crate::{
    actions::{Tweak, TweakAction, TweakStatus},
    tweaks::TweakId,
};

/// Defines the types of messages the worker can receive.
#[derive(Clone, Debug)]
pub enum WorkerMessage {
    ApplyTweak { tweak: Arc<Mutex<Tweak>> },
    RevertTweak { tweak: Arc<Mutex<Tweak>> },
    Shutdown,
}

/// Defines the types of results the worker can send back.
#[derive(Clone, Debug)]
pub enum WorkerResult {
    TweakCompleted {
        id: TweakId,
        success: bool,
        error: Option<String>,
    },
}

/// The worker responsible for processing tweak actions asynchronously.
pub struct TweakWorker {
    pub sender: Sender<WorkerMessage>,
    pub receiver: Receiver<WorkerResult>,
}

impl TweakWorker {
    /// Creates a new `TweakWorker`, initializing the communication channels and spawning the worker thread.
    pub fn new() -> Self {
        let (command_sender, command_receiver) = unbounded::<WorkerMessage>();
        let (result_sender, result_receiver) = unbounded::<WorkerResult>();

        // Spawn the worker thread
        thread::spawn(move || {
            worker_loop(command_receiver, result_sender);
        });

        TweakWorker {
            sender: command_sender,
            receiver: result_receiver,
        }
    }
}

/// The main loop of the worker thread, processing incoming messages.
fn worker_loop(command_receiver: Receiver<WorkerMessage>, result_sender: Sender<WorkerResult>) {
    loop {
        match command_receiver.recv() {
            Ok(message) => {
                match message {
                    WorkerMessage::ApplyTweak { tweak } => {
                        // Acquire the lock on the tweak
                        let mut tweak_guard = tweak.lock().expect("Failed to lock the tweak mutex");

                        // Update the status to 'Applying'
                        tweak_guard.status = TweakStatus::Applying;

                        // Attempt to apply the tweak
                        match tweak_guard.apply() {
                            Ok(_) => {
                                // On success, update the enabled state and status
                                tweak_guard
                                    .enabled
                                    .store(true, std::sync::atomic::Ordering::SeqCst);
                                tweak_guard.status = TweakStatus::Idle;
                                let _ = result_sender.send(WorkerResult::TweakCompleted {
                                    id: tweak_guard.id,
                                    success: true,
                                    error: None,
                                });
                            }
                            Err(e) => {
                                // On failure, update the status with the error
                                tweak_guard.status =
                                    TweakStatus::Failed(format!("Apply failed: {}", e));
                                let _ = result_sender.send(WorkerResult::TweakCompleted {
                                    id: tweak_guard.id,
                                    success: false,
                                    error: Some(format!("Apply failed: {}", e)),
                                });
                            }
                        }
                    }
                    WorkerMessage::RevertTweak { tweak } => {
                        // Acquire the lock on the tweak
                        let mut tweak_guard = tweak.lock().expect("Failed to lock the tweak mutex");

                        // Update the status to 'Applying'
                        tweak_guard.status = TweakStatus::Applying;

                        // Attempt to revert the tweak
                        match tweak_guard.revert() {
                            Ok(_) => {
                                // On success, update the enabled state and status
                                tweak_guard
                                    .enabled
                                    .store(false, std::sync::atomic::Ordering::SeqCst);
                                tweak_guard.status = TweakStatus::Idle;
                                let _ = result_sender.send(WorkerResult::TweakCompleted {
                                    id: tweak_guard.id,
                                    success: true,
                                    error: None,
                                });
                            }
                            Err(e) => {
                                // On failure, update the status with the error
                                tweak_guard.status =
                                    TweakStatus::Failed(format!("Revert failed: {}", e));
                                let _ = result_sender.send(WorkerResult::TweakCompleted {
                                    id: tweak_guard.id,
                                    success: false,
                                    error: Some(format!("Revert failed: {}", e)),
                                });
                            }
                        }
                    }
                    WorkerMessage::Shutdown => {
                        // Handle any necessary cleanup here
                        tracing::info!("Worker received Shutdown message. Terminating.");
                        break;
                    }
                }
            }
            Err(_) => {
                // If the channel is closed, terminate the worker
                tracing::info!("Command channel closed. Worker terminating.");
                break;
            }
        }
    }
}
