// src/main.rs

mod actions;
mod errors;
mod tweaks;
mod widgets;
mod worker;

use std::sync::{atomic, Arc, Mutex};

use actions::{Tweak, TweakAction};
use eframe::{egui, App, Frame, NativeOptions};
use egui::Widget;
use widgets::{button::ButtonState, switch::ToggleSwitchState, TweakWidget};

use crate::{
    actions::TweakStatus,
    tweaks::initialize_all_tweaks,
    widgets::{button::ApplyButton, switch::ToggleSwitch},
    worker::{TweakExecutor, WorkerMessage, WorkerResult},
};

struct MyApp {
    tweaks: Vec<Arc<Mutex<Tweak>>>,
    executor: TweakExecutor,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize tweaks
        tracing::info!("Initializing tweaks...");
        let tweaks = initialize_all_tweaks();

        // Initialize tweak executor
        tracing::info!("Initializing tweak executor...");
        let executor = TweakExecutor::new();

        // Initialize the current state of all tweaks
        for tweak_arc in &tweaks {
            let mut tweak = tweak_arc.lock().unwrap();
            match tweak.is_enabled() {
                Ok(enabled) => {
                    tweak.enabled.store(enabled, atomic::Ordering::SeqCst);
                    tweak.status = TweakStatus::Idle;
                }
                Err(e) => {
                    tweak.status = TweakStatus::Failed(format!("Initialization error: {}", e));
                    tracing::error!("Failed to initialize tweak {}: {}", tweak.name, e);
                }
            }
        }

        Self { tweaks, executor }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Poll for worker results
        while let Ok(result) = self.executor.receiver.try_recv() {
            match result {
                WorkerResult::TweakCompleted { id, success, error } => {
                    tracing::info!(
                        "Processing WorkerResult for tweak ID: {:?}, success: {:?}, error: {:?}",
                        id,
                        success,
                        error
                    );
                    if let Some(tweak_arc) = self.tweaks.iter().find(|t| {
                        let t_lock = t.lock().unwrap();
                        t_lock.id == id
                    }) {
                        let mut tweak = tweak_arc.lock().unwrap();
                        if success {
                            tweak.status = TweakStatus::Idle;
                            // Toggle the enabled state if it's a toggle tweak
                            if let TweakWidget::Switch = tweak.widget {
                                // Since the tweak was successfully applied or reverted,
                                // update the enabled flag based on the action performed
                                let new_state = if tweak.enabled.load(atomic::Ordering::SeqCst) {
                                    // If it was enabled, it was just disabled, and vice versa
                                    false
                                } else {
                                    true
                                };
                                tweak.enabled.store(new_state, atomic::Ordering::SeqCst);
                                tracing::info!(
                                    "Tweak ID: {:?} enabled state set to {:?}",
                                    id,
                                    new_state
                                );
                            }
                        } else {
                            tweak.status = TweakStatus::Failed(
                                error.unwrap_or_else(|| "Unknown error".to_string()),
                            );
                            tracing::warn!(
                                "Tweak ID: {:?} failed with error: {:?}",
                                id,
                                tweak.status
                            );
                        }
                    } else {
                        tracing::warn!("Received WorkerResult for unknown tweak ID: {:?}", id);
                    }
                }
            }
        }

        // Render the UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Overclocking Assistant");
            ui.separator();

            for tweak_arc in &self.tweaks {
                // Clone necessary data to avoid holding the lock during UI operations
                let tweak = tweak_arc.lock().unwrap().clone();

                ui.horizontal(|ui| {
                    // Tweak Information
                    ui.vertical(|ui| {
                        ui.label(tweak.name.to_string());
                        ui.label(&tweak.description);
                    });

                    // Tweak Widget
                    match tweak.widget {
                        TweakWidget::Switch => {
                            let current_state = match &tweak.status {
                                TweakStatus::Idle => {
                                    if tweak.enabled.load(atomic::Ordering::SeqCst) {
                                        ToggleSwitchState::On
                                    } else {
                                        ToggleSwitchState::Off
                                    }
                                }
                                TweakStatus::Applying => ToggleSwitchState::InProgress,
                                TweakStatus::Failed(_) => ToggleSwitchState::Off, // Optionally retain previous state
                            };
                            let toggle_widget = ToggleSwitch::new(current_state);
                            let response = toggle_widget.ui(ui);

                            if response.clicked() && current_state != ToggleSwitchState::InProgress
                            {
                                // Clone the Arc to move into the closure
                                let tweak_to_modify = tweak_arc.clone();

                                // Update status to Applying without holding the lock during UI closure
                                {
                                    let mut tweak_locked = tweak_to_modify.lock().unwrap();
                                    tweak_locked.status = TweakStatus::Applying;
                                }

                                // Send tweak to executor
                                let is_toggle = true;
                                if let Err(e) =
                                    self.executor.sender.send(WorkerMessage::ExecuteTweak {
                                        tweak: tweak_to_modify.clone(),
                                        is_toggle,
                                    })
                                {
                                    tracing::error!("Failed to send ExecuteTweak message: {:?}", e);
                                    // Optionally, revert the status or notify the user
                                    let mut tweak_locked = tweak_to_modify.lock().unwrap();
                                    tweak_locked.status =
                                        TweakStatus::Failed("Failed to send tweak".to_string());
                                }
                            }
                        }
                        TweakWidget::Button => {
                            let current_state = match tweak.status {
                                TweakStatus::Idle => ButtonState::Default,
                                TweakStatus::Applying => ButtonState::InProgress,
                                TweakStatus::Failed(_) => ButtonState::Default,
                            };
                            let button_widget = ApplyButton::new(current_state);
                            let response = button_widget.ui(ui);

                            if response.clicked() && current_state == ButtonState::Default {
                                // Clone the Arc to move into the closure
                                let tweak_to_modify = tweak_arc.clone();

                                // Update status to Applying without holding the lock during UI closure
                                {
                                    let mut tweak_locked = tweak_to_modify.lock().unwrap();
                                    tweak_locked.status = TweakStatus::Applying;
                                }

                                // Send tweak to executor
                                let is_toggle = false;
                                if let Err(e) =
                                    self.executor.sender.send(WorkerMessage::ExecuteTweak {
                                        tweak: tweak_to_modify.clone(),
                                        is_toggle,
                                    })
                                {
                                    tracing::error!("Failed to send ExecuteTweak message: {}", e);
                                    // Optionally, revert the status or notify the user
                                    let mut tweak_locked = tweak_to_modify.lock().unwrap();
                                    tweak_locked.status =
                                        TweakStatus::Failed("Failed to send tweak".to_string());
                                }
                            }
                        }
                    }

                    // Status Indicator
                    match &tweak.status {
                        TweakStatus::Idle => {}
                        TweakStatus::Applying => {
                            ui.label("Applying...");
                        }
                        TweakStatus::Failed(ref err) => {
                            ui.colored_label(egui::Color32::RED, format!("Failed: {}", err));
                        }
                    }
                });
                ui.separator();
            }
        });

        // Request a repaint to keep the UI responsive
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Send shutdown message to executor
        let _ = self.executor.sender.send(WorkerMessage::Shutdown);
        // Optionally, wait for executor to finish
    }
}

fn main() -> eframe::Result<()> {
    // Initialize tracing subscriber to enable logging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Overclocking Assistant...");

    // Configure eframe options as needed
    let options = NativeOptions::default();
    eframe::run_native(
        "Overclocking Assistant",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
