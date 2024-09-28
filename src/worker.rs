// src/worker.rs

use std::{
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel::{unbounded, Receiver, Sender};
use tracing::{error, info, warn};

use crate::tweaks::{Tweak, TweakId, TweakStatus};

// Define worker messages and results
#[derive(Clone, Debug)]
pub enum WorkerMessage {
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
    ShutdownComplete,
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
            Ok(message) => match message {
                WorkerMessage::ApplyTweak { tweak } => {
                    handle_apply_tweak(&tweak, &result_sender);
                }
                WorkerMessage::RevertTweak { tweak } => {
                    handle_revert_tweak(&tweak, &result_sender);
                }
                WorkerMessage::Shutdown => {
                    handle_shutdown(&result_sender);
                    break;
                }
            },
            Err(_) => {
                // Channel closed, terminate the worker
                info!("Command channel closed. Worker terminating.");
                break;
            }
        }
    }
}

/// Handles the ApplyTweak message by applying the tweak and sending the appropriate WorkerResult.
fn handle_apply_tweak(tweak: &Arc<Mutex<Tweak>>, result_sender: &Sender<WorkerResult>) {
    let tweak_clone = tweak.clone();

    // Update tweak status to Applying
    {
        let mut tweak_guard = tweak.lock().unwrap();
        tweak_guard.status = TweakStatus::Applying;
        info!("{:?} -> Applying tweak.", tweak_guard.id);
    }

    // Attempt to apply the tweak
    let apply_result = {
        let tweak_guard = tweak_clone.lock().unwrap();
        tweak_guard.apply()
    };

    // Update tweak status and pending_reboot based on the result
    match apply_result {
        Ok(_) => {
            let mut tweak_guard = tweak_clone.lock().unwrap();
            tweak_guard
                .enabled
                .store(true, std::sync::atomic::Ordering::SeqCst);
            tweak_guard.status = TweakStatus::Idle;

            if tweak_guard.requires_reboot {
                tweak_guard
                    .pending_reboot
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                info!(
                    "{:?} -> Tweak applied successfully. Pending reboot.",
                    tweak_guard.id
                );
            } else {
                info!(
                    "{:?} -> Tweak applied successfully. No reboot required.",
                    tweak_guard.id
                );
            }

            // Send TweakApplied result
            if let Err(e) = result_sender.send(WorkerResult::TweakApplied {
                id: tweak_guard.id,
                success: true,
                error: None,
            }) {
                error!("Failed to send TweakApplied result: {}", e);
            }
        }
        Err(e) => {
            let error_message = format!("{:?}", e);
            let mut tweak_guard = tweak_clone.lock().unwrap();
            tweak_guard.status = TweakStatus::Failed(error_message.clone());
            warn!(
                "{:?} -> Tweak application failed: {:?}",
                tweak_guard.id, tweak_guard.status
            );

            // Send TweakApplied result with failure
            if let Err(send_err) = result_sender.send(WorkerResult::TweakApplied {
                id: tweak_guard.id,
                success: false,
                error: Some(error_message),
            }) {
                error!("Failed to send TweakApplied failure result: {}", send_err);
            }
        }
    }
}

/// Handles the RevertTweak message by reverting the tweak and sending the appropriate WorkerResult.
fn handle_revert_tweak(tweak: &Arc<Mutex<Tweak>>, result_sender: &Sender<WorkerResult>) {
    let tweak_clone = tweak.clone();

    // Update tweak status to Applying
    {
        let mut tweak_guard = tweak.lock().unwrap();
        tweak_guard.status = TweakStatus::Applying;
        info!("{:?} -> Reverting tweak.", tweak_guard.id);
    }

    // Attempt to revert the tweak
    let revert_result = {
        let tweak_guard = tweak_clone.lock().unwrap();
        tweak_guard.revert()
    };

    // Update tweak status and pending_reboot based on the result
    match revert_result {
        Ok(_) => {
            let mut tweak_guard = tweak_clone.lock().unwrap();
            tweak_guard
                .enabled
                .store(false, std::sync::atomic::Ordering::SeqCst);
            tweak_guard.status = TweakStatus::Idle;

            if tweak_guard.requires_reboot {
                tweak_guard
                    .pending_reboot
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                info!(
                    "{:?} -> Tweak reverted successfully. Pending reboot cleared.",
                    tweak_guard.id
                );
            } else {
                info!(
                    "{:?} -> Tweak reverted successfully. No reboot required.",
                    tweak_guard.id
                );
            }

            // Send TweakReverted result
            if let Err(e) = result_sender.send(WorkerResult::TweakReverted {
                id: tweak_guard.id,
                success: true,
                error: None,
            }) {
                error!("Failed to send TweakReverted result: {}", e);
            }
        }
        Err(e) => {
            let error_message = format!("{:?}", e);
            let mut tweak_guard = tweak_clone.lock().unwrap();
            tweak_guard.status = TweakStatus::Failed(error_message.clone());
            warn!(
                "{:?} -> Tweak reversion failed: {:?}",
                tweak_guard.id, tweak_guard.status
            );

            // Send TweakReverted result with failure
            if let Err(send_err) = result_sender.send(WorkerResult::TweakReverted {
                id: tweak_guard.id,
                success: false,
                error: Some(error_message),
            }) {
                error!("Failed to send TweakReverted failure result: {}", send_err);
            }
        }
    }
}

/// Handles the Shutdown message by performing any necessary cleanup and sending ShutdownComplete.
fn handle_shutdown(result_sender: &Sender<WorkerResult>) {
    info!("Worker received Shutdown message. Terminating.");

    // Perform any necessary cleanup here (if applicable)

    // Send ShutdownComplete result
    if let Err(e) = result_sender.send(WorkerResult::ShutdownComplete) {
        error!("Failed to send ShutdownComplete result: {}", e);
    }
}
