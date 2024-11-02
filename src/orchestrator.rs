// src/worker.rs

use std::{sync::Arc, thread};

use anyhow::Error;
use crossbeam::channel;

use crate::tweaks::{TweakId, TweakMethod, TweakOption};

/// Represents the result of a processed task.
#[derive(Debug)]
pub struct TweakResult {
    pub id: TweakId,
    pub success: bool,
    pub error: Option<Error>,
    pub action: TweakAction,
    pub state: Option<TweakOption>,
}

/// Represents a task to be processed.
pub struct TweakTask {
    pub id: TweakId,
    pub method: Arc<dyn TweakMethod>,
    pub action: TweakAction,
}

/// Actions that can be performed on a tweak.
#[derive(Debug, Clone)]
pub enum TweakAction {
    Enable,
    Disable,
    Set(TweakOption),
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
    pub fn submit_task(&self, task: TweakTask) -> anyhow::Result<()> {
        let result_sender = self.result_sender.clone();
        thread::spawn(move || {
            let result = match task.action {
                TweakAction::Enable => match task.method.apply(TweakOption::Enabled(true)) {
                    Ok(_) => TweakResult {
                        id: task.id,
                        success: true,
                        error: None,
                        action: TweakAction::Enable,
                        state: Some(TweakOption::Enabled(true)),
                    },
                    Err(e) => TweakResult {
                        id: task.id,
                        success: false,
                        error: Some(e),
                        action: TweakAction::Enable,
                        state: None,
                    },
                },
                TweakAction::Disable => match task.method.revert() {
                    Ok(_) => TweakResult {
                        id: task.id,
                        success: true,
                        error: None,
                        action: TweakAction::Disable,
                        state: Some(TweakOption::Enabled(false)),
                    },
                    Err(e) => TweakResult {
                        id: task.id,
                        success: false,
                        error: Some(e),
                        action: TweakAction::Disable,
                        state: None,
                    },
                },
                TweakAction::Set(option) => match task.method.apply(option.clone()) {
                    Ok(_) => TweakResult {
                        id: task.id,
                        success: true,
                        error: None,
                        action: TweakAction::Set(option.clone()),
                        state: Some(option),
                    },
                    Err(e) => TweakResult {
                        id: task.id,
                        success: false,
                        error: Some(e),
                        action: TweakAction::Set(option),
                        state: None,
                    },
                },
                TweakAction::ReadInitialState => match task.method.initial_state() {
                    Ok(state) => TweakResult {
                        id: task.id,
                        success: true,
                        error: None,
                        action: TweakAction::ReadInitialState,
                        state: Some(state),
                    },
                    Err(e) => TweakResult {
                        id: task.id,
                        success: false,
                        error: Some(e),
                        action: TweakAction::ReadInitialState,
                        state: None,
                    },
                },
            };
            if let Err(e) = result_sender.send(result) {
                tracing::error!("Failed to send result: {:?}", e);
            }
        });
        Ok(())
    }

    /// Attempts to receive a task result without blocking.
    pub fn try_recv_result(&self) -> Option<TweakResult> {
        self.result_receiver.try_recv().ok()
    }
}
