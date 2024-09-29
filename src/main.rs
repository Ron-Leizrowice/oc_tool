// src/main.rs

mod errors;
mod tweaks;
mod utils;
mod widgets;
mod worker;

use std::sync::{atomic, Arc, Mutex};

use eframe::{egui, App, Frame, NativeOptions};
use tinyfiledialogs::YesNo;
use tracing::{debug, error, info, span, trace, warn, Level};
use tweaks::{initialize_all_tweaks, Tweak, TweakStatus};
use utils::{is_elevated, reboot_system};
use widgets::{switch::toggle_switch, TweakWidget};

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
        // Categorize tweaks
        let mut toggle_tweaks_requires_reboot = Vec::new();
        let mut toggle_tweaks_no_reboot = Vec::new();
        let mut apply_once_tweaks_requires_reboot = Vec::new();
        let mut apply_once_tweaks_no_reboot = Vec::new();

        for tweak in &self.tweaks {
            let tweak_guard = tweak.lock().unwrap();
            match tweak_guard.widget {
                TweakWidget::Switch => {
                    if tweak_guard.requires_reboot {
                        toggle_tweaks_requires_reboot.push(tweak.clone());
                    } else {
                        toggle_tweaks_no_reboot.push(tweak.clone());
                    }
                }
                TweakWidget::Button => {
                    if tweak_guard.requires_reboot {
                        apply_once_tweaks_requires_reboot.push(tweak.clone());
                    } else {
                        apply_once_tweaks_no_reboot.push(tweak.clone());
                    }
                }
            }
        }

        // Render Apply-Once Tweaks that require reboot
        if !apply_once_tweaks_requires_reboot.is_empty() {
            ui.separator();
            egui::Grid::new("apply_once_requires_reboot_grid")
                .spacing(egui::vec2(20.0, 10.0))
                .striped(true)
                .show(ui, |ui| {
                    for tweak in &apply_once_tweaks_requires_reboot {
                        let tweak_guard = tweak.lock().unwrap();

                        // Tweak Name
                        ui.label(&tweak_guard.name);

                        // Apply Button
                        let button = egui::Button::new("Apply");
                        let response =
                            ui.add_enabled(tweak_guard.status != TweakStatus::Applying, button);

                        if response.clicked() && tweak_guard.status == TweakStatus::Idle {
                            if let Err(e) = self.executor.sender.send(WorkerMessage::ApplyTweak {
                                tweak: tweak.clone(),
                            }) {
                                error!("Failed to send ApplyTweak message: {:?}", e);
                            }
                        }

                        // Display "Applying..." status
                        if tweak_guard.status == TweakStatus::Applying {
                            ui.label("Applying...");
                        }

                        // Display error messages
                        if let TweakStatus::Failed(ref err) = tweak_guard.status {
                            ui.colored_label(egui::Color32::RED, format!("Error: {:?}", err));
                        }

                        ui.end_row();
                    }
                });
        }

        // Render Apply-Once Tweaks that do not require reboot
        if !apply_once_tweaks_no_reboot.is_empty() {
            ui.separator();
            egui::Grid::new("apply_once_no_reboot_grid")
                .spacing(egui::vec2(20.0, 10.0))
                .striped(true)
                .show(ui, |ui| {
                    for tweak in &apply_once_tweaks_no_reboot {
                        let tweak_guard = tweak.lock().unwrap();

                        // Tweak Name with tooltip
                        ui.label(
                            egui::RichText::new(&tweak_guard.name)
                                .text_style(egui::TextStyle::Body),
                        )
                        .on_hover_text(tweak_guard.description.clone());

                        // Apply Button
                        let button = egui::Button::new("Apply");
                        let response =
                            ui.add_enabled(tweak_guard.status != TweakStatus::Applying, button);

                        if response.clicked() && tweak_guard.status == TweakStatus::Idle {
                            if let Err(e) = self.executor.sender.send(WorkerMessage::ApplyTweak {
                                tweak: tweak.clone(),
                            }) {
                                error!("Failed to send ApplyTweak message: {:?}", e);
                            }
                        }

                        // Display "Applying..." status
                        if tweak_guard.status == TweakStatus::Applying {
                            ui.label("Applying...");
                        }

                        // Display error messages
                        if let TweakStatus::Failed(ref err) = tweak_guard.status {
                            ui.colored_label(egui::Color32::RED, format!("Error: {:?}", err));
                        }

                        ui.end_row();
                    }
                });
        }

        // Render Toggle Tweaks that require reboot
        if !toggle_tweaks_requires_reboot.is_empty() {
            ui.separator();
            egui::Grid::new("toggle_requires_reboot_grid")
                .spacing(egui::vec2(20.0, 10.0))
                .striped(true)
                .show(ui, |ui| {
                    for tweak in &toggle_tweaks_requires_reboot {
                        let tweak_guard = tweak.lock().unwrap();

                        // Tweak Name with tooltip
                        ui.label(
                            egui::RichText::new(&tweak_guard.name)
                                .text_style(egui::TextStyle::Body),
                        )
                        .on_hover_text(tweak_guard.description.clone());

                        // Toggle Switch bound to the tweak's enabled state
                        let mut state = tweak_guard.enabled.load(atomic::Ordering::SeqCst);
                        let response = ui.add(toggle_switch(&mut state));
                        if response.changed() {
                            tweak_guard.enabled.store(state, atomic::Ordering::SeqCst);
                        }

                        // If the toggle state changed, send a message to the worker
                        if response.changed() {
                            // Create and send the appropriate message based on the new state
                            let message = if tweak_guard.enabled.load(atomic::Ordering::SeqCst) {
                                WorkerMessage::ApplyTweak {
                                    tweak: tweak.clone(),
                                }
                            } else {
                                WorkerMessage::RevertTweak {
                                    tweak: tweak.clone(),
                                }
                            };
                            if let Err(e) = self.executor.sender.send(message) {
                                error!("Failed to send tweak message: {:?}", e);
                            }
                        }

                        // Display error messages
                        if let TweakStatus::Failed(ref err) = tweak_guard.status {
                            ui.colored_label(egui::Color32::RED, format!("Error: {:?}", err));
                        }

                        ui.end_row();
                    }
                });
        }

        // Render Toggle Tweaks that do not require reboot
        if !toggle_tweaks_no_reboot.is_empty() {
            ui.separator();
            egui::Grid::new("toggle_no_reboot_grid")
                .spacing(egui::vec2(20.0, 10.0))
                .striped(true)
                .show(ui, |ui| {
                    for tweak in &toggle_tweaks_no_reboot {
                        let tweak_guard = tweak.lock().unwrap();

                        // Tweak Name with tooltip
                        ui.label(
                            egui::RichText::new(&tweak_guard.name)
                                .text_style(egui::TextStyle::Body),
                        )
                        .on_hover_text(tweak_guard.description.clone());

                        // Toggle Switch
                        let mut state = tweak_guard.enabled.load(atomic::Ordering::SeqCst);
                        let response = ui.add(toggle_switch(&mut state));

                        // If the toggle state changed, send a message to the worker
                        if response.changed() {
                            // Store the updated state back into AtomicBool
                            tweak_guard.enabled.store(state, atomic::Ordering::SeqCst);

                            // Create and send the appropriate message based on the new state
                            let message = if state {
                                WorkerMessage::ApplyTweak {
                                    tweak: tweak.clone(),
                                }
                            } else {
                                WorkerMessage::RevertTweak {
                                    tweak: tweak.clone(),
                                }
                            };
                            if let Err(e) = self.executor.sender.send(message) {
                                error!("Failed to send tweak message: {:?}", e);
                            }
                        }

                        // Display error messages
                        if let TweakStatus::Failed(ref err) = tweak_guard.status {
                            ui.colored_label(egui::Color32::RED, format!("Error: {:?}", err));
                        }

                        ui.end_row();
                    }
                });
        }
    }

    /// Renders the status bar at the bottom with divisions.
    fn draw_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(10.0); // Add some vertical padding

            ui.horizontal(|ui| {
                // First Division: General status
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
                        // Show confirmation dialog
                        if tinyfiledialogs::message_box_yes_no(
                            "Confirm Reboot",
                            "Are you sure you want to reboot the system now?",
                            tinyfiledialogs::MessageBoxIcon::Question,
                            YesNo::Yes,
                        ) == YesNo::Yes
                        {
                            // Trigger system reboot
                            if let Err(e) = reboot_system() {
                                error!("Failed to initiate reboot: {:?}", e);
                                tinyfiledialogs::message_box_ok(
                                    "Overclocking Assistant",
                                    &format!("Failed to reboot the system: {:?}", e),
                                    tinyfiledialogs::MessageBoxIcon::Error,
                                );
                            }
                        }
                    }
                }

                ui.separator(); // Vertical separator

                // Third Division: Additional info
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
            error!("Failed to send Shutdown message: {:?}", e);
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
                "OC Tool",
                "Administrator privileges required",
                tinyfiledialogs::MessageBoxIcon::Error,
            );
            return Ok(());
        }
    }

    // Initialize tracing subscriber to enable logging with more detailed settings
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    info!("Starting Overclocking Assistant...");

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 1000.0]) // Set a default initial size
            .with_min_inner_size([400.0, 300.0]), // Set a minimum size
        ..Default::default()
    };

    // Create a tracing span for the run_native call
    let run_span = span!(Level::INFO, "Run Native");
    run_span.in_scope(|| {
        trace!("Entering Run Native span.");
        eframe::run_native(
            "OC Tool",
            options,
            Box::new(|cc| {
                tracing::debug!("Creating MyApp instance.");
                Ok(Box::new(MyApp::new(cc)))
            }),
        )
    })
}
