// src/worker.rs

use std::{sync::Arc, thread, time::Duration};

use crossbeam::channel;
use tracing::{error, info};

use crate::tweaks::{TweakId, TweakMethod};

/// Represents the result of a processed task.
#[derive(Debug, Clone)]
pub struct TweakResult {
    pub id: TweakId,
    pub success: bool,
    pub error: Option<String>,
    pub enabled_state: Option<bool>, // Some(true) if enabled, Some(false) if disabled, None if unknown
    pub action: TweakAction,
}

/// Represents a task to be processed.
#[derive(Clone)]
pub struct TweakTask {
    pub id: TweakId,
    pub method: Arc<dyn TweakMethod>,
    pub action: TweakAction,
}

/// Actions that can be performed on a tweak.
#[derive(Debug, Clone)]
pub enum TweakAction {
    Apply,
    Revert,
    ReadInitialState,
}

/// The orchestrator managing task execution.
pub struct TaskOrchestrator {
    result_receiver: channel::Receiver<TweakResult>,
    result_sender: channel::Sender<TweakResult>,
}

impl Default for TaskOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskOrchestrator {
    /// Creates a new TaskOrchestrator.
    pub fn new() -> Self {
        let (result_sender, result_receiver) = channel::unbounded::<TweakResult>();
        Self {
            result_sender,
            result_receiver,
        }
    }

    /// Submits a new task to be processed.
    pub fn submit_task(&self, task: TweakTask) -> Result<(), String> {
        let result_sender = self.result_sender.clone();
        thread::spawn(move || {
            info!("Processing task {:?}", task.id);
            let result = match task.action {
                TweakAction::Apply => {
                    info!("Applying tweak {:?}", task.id);
                    match task.method.apply() {
                        Ok(_) => TweakResult {
                            id: task.id,
                            success: true,
                            error: None,
                            enabled_state: Some(true),
                            action: TweakAction::Apply,
                        },
                        Err(e) => TweakResult {
                            id: task.id,
                            success: false,
                            error: Some(e.to_string()),
                            enabled_state: None,
                            action: TweakAction::Apply,
                        },
                    }
                }
                TweakAction::Revert => {
                    info!("Reverting tweak {:?}", task.id);
                    match task.method.revert() {
                        Ok(_) => TweakResult {
                            id: task.id,
                            success: true,
                            error: None,
                            enabled_state: Some(false),
                            action: TweakAction::Revert,
                        },
                        Err(e) => TweakResult {
                            id: task.id,
                            success: false,
                            error: Some(e.to_string()),
                            enabled_state: None,
                            action: TweakAction::Revert,
                        },
                    }
                }
                TweakAction::ReadInitialState => {
                    info!("Reading initial state for tweak {:?}", task.id);
                    match task.method.initial_state() {
                        Ok(state) => TweakResult {
                            id: task.id,
                            success: true,
                            error: None,
                            enabled_state: Some(state),
                            action: TweakAction::ReadInitialState,
                        },
                        Err(e) => TweakResult {
                            id: task.id,
                            success: false,
                            error: Some(e.to_string()),
                            enabled_state: None,
                            action: TweakAction::ReadInitialState,
                        },
                    }
                }
            };
            if let Err(e) = result_sender.send(result) {
                error!("Failed to send result: {:?}", e);
            }
            // Simulate processing time
            thread::sleep(Duration::from_millis(100));
        });
        Ok(())
    }

    /// Attempts to receive a task result without blocking.
    pub fn try_recv_result(&self) -> Option<TweakResult> {
        self.result_receiver.try_recv().ok()
    }
}
