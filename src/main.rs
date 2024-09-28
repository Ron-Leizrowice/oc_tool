// src/main.rs

mod errors;
mod tweaks;
mod utils;
mod widgets;
mod worker;
use std::sync::{atomic, Arc, Mutex};

use eframe::{egui, App, Frame, NativeOptions};
use tracing::{debug, error, info, span, trace, warn, Level};
use tweaks::{initialize_all_tweaks, Tweak, TweakStatus};
use utils::is_elevated;
use widgets::{
    switch::{ToggleSwitch, ToggleSwitchState},
    TweakWidget,
};

use crate::worker::{TweakWorker, WorkerMessage, WorkerResult};

struct MyApp {
    tweaks: Vec<Arc<Mutex<Tweak>>>,
    executor: TweakWorker,
}

impl MyApp {
    /// Initializes the application by setting up tweaks and the worker.
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize tracing spans for better context
        let app_span = span!(Level::INFO, "App Initialization");
        let _app_guard = app_span.enter();

        info!("Initializing Overclocking Assistant application");

        // Initialize tweaks
        info!("Initializing tweaks...");
        let tweaks = initialize_all_tweaks();

        // Initialize tweak executor
        info!("Initializing tweak executor...");
        let executor = TweakWorker::new();

        // Initialize the current state of all tweaks
        info!("Checking initial state of all tweaks");
        for arc_tweak in &tweaks {
            let mut tweak = arc_tweak.lock().unwrap();
            match tweak.check_initial_state() {
                Ok(enabled) => {
                    info!(
                        "{:?} -> Initial state: {}",
                        tweak.id,
                        if enabled { "enabled" } else { "disabled" }
                    );
                    tweak
                        .enabled
                        .store(enabled, std::sync::atomic::Ordering::SeqCst);
                    tweak.status = TweakStatus::Idle;
                }
                Err(e) => {
                    warn!("{:?} -> Initialization error: {}", tweak.id, e);
                    tweak.status = TweakStatus::Failed(format!("Initialization error: {}", e));
                }
            }
        }
        info!("Application initialization complete");
        Self { tweaks, executor }
    }

    /// Processes any incoming results from the worker and updates tweak statuses.
    fn process_worker_results(&mut self) {
        while let Ok(result) = self.executor.receiver.try_recv() {
            match result {
                WorkerResult::TweakApplied { id, success, error } => {
                    if let Some(tweak) = self.tweaks.iter().find(|t| t.lock().unwrap().id == id) {
                        let mut tweak_guard = tweak.lock().unwrap();
                        if success {
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
                        } else {
                            tweak_guard.status = TweakStatus::Failed(
                                error.unwrap_or_else(|| "Unknown error".to_string()),
                            );
                            warn!(
                                "{:?} -> Tweak application failed: {:?}",
                                tweak_guard.id, tweak_guard.status
                            );
                        }
                    } else {
                        warn!("Received result for unknown tweak: {:?}", id);
                    }
                }
                WorkerResult::TweakReverted { id, success, error } => {
                    if let Some(tweak) = self.tweaks.iter().find(|t| t.lock().unwrap().id == id) {
                        let mut tweak_guard = tweak.lock().unwrap();
                        if success {
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
                        } else {
                            tweak_guard.status = TweakStatus::Failed(
                                error.unwrap_or_else(|| "Unknown error".to_string()),
                            );
                            warn!(
                                "{:?} -> Tweak reversion failed: {:?}",
                                tweak_guard.id, tweak_guard.status
                            );
                        }
                    } else {
                        warn!("Received result for unknown tweak: {:?}", id);
                    }
                }
                WorkerResult::ShutdownComplete => {
                    info!("Worker has shut down gracefully.");
                }
            }
        }
    }

    /// Renders the entire UI by iterating over all tweaks.
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.heading("Overclocking Assistant");

        for tweak in &self.tweaks {
            self.render_tweak(ui, tweak);
        }
    }

    /// Renders an individual tweak with its corresponding widget and handles interactions.
    fn render_tweak(&self, ui: &mut egui::Ui, tweak: &Arc<Mutex<Tweak>>) {
        let tweak_guard = tweak.lock().unwrap();

        // Use a vertical layout to allow the description to appear below the tweak row
        ui.vertical(|ui| {
            // Create a horizontal layout for the tweak name and widget
            let inner_response = ui.horizontal(|ui| {
                ui.label(&tweak_guard.name);

                match tweak_guard.widget {
                    TweakWidget::Switch => {
                        let is_enabled = tweak_guard
                            .enabled
                            .load(std::sync::atomic::Ordering::SeqCst);
                        let state = if tweak_guard.status == TweakStatus::Applying {
                            ToggleSwitchState::InProgress
                        } else if is_enabled {
                            ToggleSwitchState::On
                        } else {
                            ToggleSwitchState::Off
                        };

                        let toggle = ToggleSwitch::new(state);
                        let response =
                            ui.add_enabled(tweak_guard.status != TweakStatus::Applying, toggle);

                        if response.clicked() && tweak_guard.status == TweakStatus::Idle {
                            let new_state = !is_enabled;
                            if new_state {
                                // Apply the tweak
                                if let Err(e) =
                                    self.executor.sender.send(WorkerMessage::ApplyTweak {
                                        tweak: tweak.clone(),
                                    })
                                {
                                    error!("Failed to send ApplyTweak message: {}", e);
                                }
                            } else {
                                // Revert the tweak
                                if let Err(e) =
                                    self.executor.sender.send(WorkerMessage::RevertTweak {
                                        tweak: tweak.clone(),
                                    })
                                {
                                    error!("Failed to send RevertTweak message: {}", e);
                                }
                            }
                        }

                        // Display "Applying..." status
                        if tweak_guard.status == TweakStatus::Applying {
                            ui.label("Applying...");
                        }

                        // Display error messages
                        if let TweakStatus::Failed(ref err) = tweak_guard.status {
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                        }
                    }
                    TweakWidget::Button => {
                        let button = egui::Button::new("Apply");
                        let response =
                            ui.add_enabled(tweak_guard.status != TweakStatus::Applying, button);

                        if response.clicked() && tweak_guard.status == TweakStatus::Idle {
                            // Apply the one-time tweak
                            if let Err(e) = self.executor.sender.send(WorkerMessage::ApplyTweak {
                                tweak: tweak.clone(),
                            }) {
                                error!("Failed to send ApplyTweak message: {}", e);
                            }
                        }

                        // Display "Applying..." status
                        if tweak_guard.status == TweakStatus::Applying {
                            ui.label("Applying...");
                        }

                        // Display error messages
                        if let TweakStatus::Failed(ref err) = tweak_guard.status {
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                        }
                    }
                }
            });

            // Show description only when the tweak row is hovered
            if inner_response.response.hovered() {
                ui.indent("   ", |ui| {}); // Optional: Indent the description for better UI
                ui.label(&tweak_guard.description);
            }
        });
    }

    /// Renders the status bar at the bottom with divisions.
    fn draw_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(10.0); // Add some vertical padding

            ui.horizontal(|ui| {
                // First Division: Placeholder for general status
                ui.label("Status: All systems operational");

                ui.separator(); // Vertical separator

                // Second Division: Tweaks pending restart
                let pending_reboot_count = self
                    .tweaks
                    .iter()
                    .filter(|t| {
                        t.lock()
                            .unwrap()
                            .pending_reboot
                            .load(atomic::Ordering::SeqCst)
                    })
                    .count();
                ui.label(format!(
                    "{} tweak{} pending restart",
                    pending_reboot_count,
                    if pending_reboot_count != 1 { "s" } else { "" }
                ));

                // If there are pending reboots, show a reboot button
                if pending_reboot_count > 0 {
                    ui.separator(); // Vertical separator
                    if ui.button("Reboot Now").clicked() {
                        // Trigger system reboot
                        if let Err(e) = utils::reboot_system() {
                            error!("Failed to initiate reboot: {}", e);
                            tinyfiledialogs::message_box_ok(
                                "Overclocking Assistant",
                                &format!("Failed to reboot the system: {}", e),
                                tinyfiledialogs::MessageBoxIcon::Error,
                            );
                        }
                    }
                }

                ui.separator(); // Vertical separator

                // Third Division: Placeholder for additional info
                ui.label("Version: 1.0.0");
            });

            ui.add_space(10.0); // Add some vertical padding
        });
    }
}

impl App for MyApp {
    /// The main update loop where the UI is rendered and interactions are handled.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Process any incoming results from the worker
        self.process_worker_results();

        // Render the main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_ui(ui);
        });

        // Render the status bar
        self.draw_status_bar(ctx);
    }

    /// Handles application exit by sending a shutdown message to the worker.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("Application is exiting. Sending shutdown message to executor.");
        // Send shutdown message to executor
        if let Err(e) = self.executor.sender.send(WorkerMessage::Shutdown) {
            error!(
                error = ?e,
                "Failed to send Shutdown message."
            );
        } else {
            debug!("Shutdown message sent to executor.");
        }
        info!("Shutdown process complete.");
    }
}

fn main() -> eframe::Result<()> {
    match is_elevated() {
        true => info!("Running with elevated privileges."),
        false => {
            tinyfiledialogs::message_box_ok(
                "Overclocking Assistant",
                "Administrator privileges required",
                tinyfiledialogs::MessageBoxIcon::Error,
            );
            return Ok(());
        }
    }

    // Initialize tracing subscriber to enable logging with more detailed settings
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG) // Set the maximum log level
        .with_target(false) // Optionally hide the log target (module path)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE) // Log when spans close
        .init();

    info!("Starting Overclocking Assistant...");

    // Configure eframe options as needed
    let options = NativeOptions::default();

    // Create a tracing span for the run_native call
    let run_span = span!(Level::INFO, "Run Native");
    run_span.in_scope(|| {
        trace!("Entering Run Native span.");
        eframe::run_native(
            "Overclocking Assistant",
            options,
            Box::new(|cc| {
                tracing::debug!("Creating MyApp instance.");
                Ok(Box::new(MyApp::new(cc)))
            }),
        )
    })
}
