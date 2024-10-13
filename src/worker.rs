// src/worker.rs

use std::{sync::Arc, thread, time::Duration};

use crossbeam::channel;
use tracing::{error, info};

use crate::tweaks::{TweakId, TweakMethod};

const NUM_WORKERS: usize = 8;

/// Commands that can be sent to a worker.
#[derive(Clone)]
pub enum WorkerCommand {
    ApplyTweak {
        id: TweakId,
        method: Arc<dyn TweakMethod>,
    },
    RevertTweak {
        id: TweakId,
        method: Arc<dyn TweakMethod>,
    },
    ReadTweakState {
        id: TweakId,
        method: Arc<dyn TweakMethod>,
    },
    Shutdown,
}

/// Represents the result of a processed command.
#[derive(Debug, Clone)]
pub struct TweakResult {
    pub id: TweakId,
    pub success: bool,
    pub error: Option<String>,
    pub enabled_state: Option<bool>, // Some(true) if enabled, Some(false) if disabled, None if unknown
    pub action: TweakAction,
}

/// Represents a worker thread that processes `TweakTask`s.
pub struct Worker {
    id: usize,
    command_receiver: channel::Receiver<WorkerCommand>,
    result_sender: channel::Sender<TweakResult>,
}

impl Worker {
    pub fn new(
        id: usize,
        command_receiver: channel::Receiver<WorkerCommand>,
        result_sender: channel::Sender<TweakResult>,
    ) -> Self {
        Self {
            id,
            command_receiver,
            result_sender,
        }
    }

    pub fn run(self) -> thread::JoinHandle<()> {
        let worker_id = self.id;
        let command_receiver = self.command_receiver;
        let result_sender = self.result_sender;
        thread::spawn(move || {
            info!("Worker {} started.", worker_id);
            while let Ok(command) = command_receiver.recv() {
                match command {
                    WorkerCommand::ApplyTweak { id, method } => {
                        info!("Worker {} applying tweak {:?}", worker_id, id);
                        let result = match method.apply(id) {
                            Ok(_) => TweakResult {
                                id,
                                success: true,
                                error: None,
                                enabled_state: Some(true),
                                action: TweakAction::Apply,
                            },
                            Err(e) => TweakResult {
                                id,
                                success: false,
                                error: Some(e.to_string()),
                                enabled_state: None,
                                action: TweakAction::Apply,
                            },
                        };
                        if let Err(e) = result_sender.send(result) {
                            error!("Worker {} failed to send result: {:?}", worker_id, e);
                            break;
                        }
                    }
                    WorkerCommand::RevertTweak { id, method } => {
                        info!("Worker {} reverting tweak {:?}", worker_id, id);
                        let result = match method.revert(id) {
                            Ok(_) => TweakResult {
                                id,
                                success: true,
                                error: None,
                                enabled_state: Some(false),
                                action: TweakAction::Revert,
                            },
                            Err(e) => TweakResult {
                                id,
                                success: false,
                                error: Some(e.to_string()),
                                enabled_state: None,
                                action: TweakAction::Revert,
                            },
                        };
                        if let Err(e) = result_sender.send(result) {
                            error!("Worker {} failed to send result: {:?}", worker_id, e);
                            break;
                        }
                    }
                    WorkerCommand::ReadTweakState { id, method } => {
                        info!(
                            "Worker {} reading initial state for tweak {:?}",
                            worker_id, id
                        );
                        let result = match method.initial_state(id) {
                            Ok(state) => TweakResult {
                                id,
                                success: true,
                                error: None,
                                enabled_state: Some(state),
                                action: TweakAction::ReadInitialState,
                            },
                            Err(e) => TweakResult {
                                id,
                                success: false,
                                error: Some(e.to_string()),
                                enabled_state: None,
                                action: TweakAction::ReadInitialState,
                            },
                        };
                        if let Err(e) = result_sender.send(result) {
                            error!("Worker {} failed to send result: {:?}", worker_id, e);
                            break;
                        }
                    }
                    WorkerCommand::Shutdown => {
                        info!("Worker {} received shutdown command.", worker_id);
                        break;
                    }
                }
                // Simulate processing time
                thread::sleep(Duration::from_millis(100));
            }
            info!("Worker {} shutting down.", worker_id);
        })
    }
}

/// Represents a task to be processed by a worker.
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

/// The orchestrator managing worker threads and task distribution.
pub struct TaskOrchestrator {
    command_sender: channel::Sender<WorkerCommand>,
    result_receiver: channel::Receiver<TweakResult>,
    workers: Vec<WorkerHandle>,
}

struct WorkerHandle {
    handle: Option<thread::JoinHandle<()>>, // Wrap in Option to allow moving out
}

impl Default for TaskOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskOrchestrator {
    /// Creates a new TaskOrchestrator with a specified number of workers.
    pub fn new() -> Self {
        let (command_sender, command_receiver) = channel::unbounded::<WorkerCommand>();
        let (result_sender, result_receiver) = channel::unbounded::<TweakResult>();

        let mut workers = Vec::with_capacity(NUM_WORKERS);

        for i in 0..NUM_WORKERS {
            let command_receiver_clone = command_receiver.clone();
            let result_sender_clone = result_sender.clone();
            let worker = Worker::new(i, command_receiver_clone, result_sender_clone);
            let handle = worker.run();
            workers.push(WorkerHandle {
                handle: Some(handle),
            });
        }

        // Drop extra clones to allow workers to exit
        drop(command_receiver);
        drop(result_sender);

        Self {
            command_sender,
            result_receiver,
            workers,
        }
    }

    /// Submits a new task to be processed by the workers.
    pub fn submit_task(&self, task: TweakTask) -> Result<(), channel::SendError<WorkerCommand>> {
        let command = match task.action {
            TweakAction::Apply => WorkerCommand::ApplyTweak {
                id: task.id,
                method: task.method,
            },
            TweakAction::Revert => WorkerCommand::RevertTweak {
                id: task.id,
                method: task.method,
            },
            TweakAction::ReadInitialState => WorkerCommand::ReadTweakState {
                id: task.id,
                method: task.method,
            },
        };
        self.command_sender.send(command)
    }

    /// Attempts to receive a task result without blocking.
    pub fn try_recv_result(&self) -> Option<TweakResult> {
        self.result_receiver.try_recv().ok()
    }

    /// Gracefully shuts down all workers by sending a shutdown command.
    pub fn shutdown(&mut self) {
        // Send a Shutdown command to each worker
        for _ in &self.workers {
            let _ = self.command_sender.send(WorkerCommand::Shutdown);
        }

        // Wait for workers to finish
        for worker in &mut self.workers {
            // Take the handle out of the Option
            if let Some(handle) = worker.handle.take() {
                if let Err(e) = handle.join() {
                    error!("Failed to join worker thread: {:?}", e);
                }
            }
        }
    }
}
