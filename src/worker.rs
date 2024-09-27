// src/worker.rs

use std::{
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
            while let Ok(message) = task_receiver.recv() {
                match message {
                    WorkerMessage::ExecuteTweak {
                        tweak: tweak_arc,
                        is_toggle,
                    } => {
                        let tweak_id = {
                            let tweak_guard = tweak_arc.lock().unwrap();
                            tweak_guard.id
                        };

                        tracing::info!(
                            "Starting tweak {:?} as {}",
                            tweak_id,
                            if is_toggle { "Toggle" } else { "Apply" }
                        );

                        let result = {
                            let tweak = (*tweak_arc.lock().unwrap()).clone();
                            if is_toggle {
                                match tweak.clone().is_enabled() {
                                    Ok(enabled) => {
                                        if enabled {
                                            tweak.revert()
                                        } else {
                                            tweak.apply()
                                        }
                                    }
                                    Err(e) => Err(e),
                                }
                            } else {
                                tweak.apply()
                            }
                        };

                        match result {
                            Ok(_) => {
                                tracing::info!("Tweak {:?} completed successfully.", tweak_id);
                                let _ = result_sender.send(WorkerResult::TweakCompleted {
                                    id: tweak_id,
                                    success: true,
                                    error: None,
                                });
                            }
                            Err(e) => {
                                tracing::error!("Failed to execute tweak {:?}: {}", tweak_id, e);
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
            tracing::info!("TweakExecutor thread terminating.");
        });

        Self {
            sender: task_sender,
            receiver: result_receiver,
        }
    }
}
