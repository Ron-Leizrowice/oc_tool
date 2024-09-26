// src/main.rs

mod actions;
mod errors;
mod tweaks;
mod widgets;
mod worker;

use std::sync::{Arc, Mutex};

use actions::{Tweak, TweakAction};
use chrono::Local;
use eframe::{egui, App, Frame, NativeOptions};
use egui::Widget;
use fern::Dispatch;
use log::LevelFilter;
use widgets::{button::ButtonState, switch::ToggleSwitchState};

use crate::{
    actions::TweakStatus,
    tweaks::initialize_all_tweaks,
    widgets::{button::ApplyButton, switch::ToggleSwitch, TweakWidget},
    worker::{TweakExecutor, WorkerMessage, WorkerResult},
};

// Setting up logging using fern
fn setup_logging() {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}][{}] {}",
                Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("app.log").unwrap())
        .apply()
        .unwrap();
}

struct MyApp {
    tweaks: Vec<Arc<Mutex<Tweak>>>,
    executor: TweakExecutor,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize logging
        setup_logging();

        // Initialize tweaks
        let tweaks = initialize_all_tweaks();

        // Initialize tweak executor
        let executor = TweakExecutor::new();

        // Optionally, you can load tweak statuses from persisted state here

        Self { tweaks, executor }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Poll for worker results
        while let Ok(result) = self.executor.receiver.try_recv() {
            match result {
                WorkerResult::TweakCompleted { id, success, error } => {
                    if let Some(tweak_arc) = self.tweaks.iter().find(|t| t.lock().unwrap().id == id)
                    {
                        let mut tweak = tweak_arc.lock().unwrap();
                        if success {
                            tweak.status = TweakStatus::Idle;
                            // Update toggle state if it's a toggle tweak
                        } else {
                            tweak.status = TweakStatus::Failed(
                                error.unwrap_or_else(|| "Unknown error".to_string()),
                            );
                        }
                    }
                }
            }
        }

        // Render the UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Overclocking Assistant");
            ui.separator();

            for tweak_arc in &self.tweaks {
                let tweak = &*tweak_arc.lock().unwrap();
                ui.horizontal(|ui| {
                    // Tweak Information
                    ui.vertical(|ui| {
                        ui.label(format!("**{}**", tweak.name));
                        ui.label(&tweak.description);
                    });

                    // Tweak Widget
                    match tweak.widget {
                        TweakWidget::Switch(_) => {
                            let is_enabled = tweak.clone().is_enabled().unwrap_or(false);
                            let current_state = match &tweak.status {
                                TweakStatus::Idle => {
                                    if is_enabled {
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
                                // Update status to Applying
                                if let Some(tweak_arc) = self
                                    .tweaks
                                    .iter()
                                    .find(|t| t.lock().unwrap().id == tweak.id)
                                {
                                    let mut tweak_locked = tweak_arc.lock().unwrap();
                                    tweak_locked.status = TweakStatus::Applying;
                                }

                                // Send tweak to executor
                                let is_toggle = true;
                                let _ = self.executor.sender.send(WorkerMessage::ExecuteTweak {
                                    tweak: tweak_arc.clone(),
                                    is_toggle,
                                });
                            }
                        }
                        TweakWidget::Button(_) => {
                            let current_state = match tweak.status {
                                TweakStatus::Idle => ButtonState::Default,
                                TweakStatus::Applying => ButtonState::InProgress,
                                TweakStatus::Failed(_) => ButtonState::Default,
                            };
                            let button_widget = ApplyButton::new(current_state);
                            let response = button_widget.ui(ui);

                            if response.clicked() && current_state == ButtonState::Default {
                                // Update status to Applying
                                if let Some(tweak_arc) = self
                                    .tweaks
                                    .iter()
                                    .find(|t| t.lock().unwrap().id == tweak.id)
                                {
                                    let mut tweak_locked = tweak_arc.lock().unwrap();
                                    tweak_locked.status = TweakStatus::Applying;
                                }

                                // Send tweak to executor

                                let is_toggle = false;
                                let _ = self.executor.sender.send(WorkerMessage::ExecuteTweak {
                                    tweak: tweak_arc.clone(),
                                    is_toggle,
                                });
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
    // Configure eframe options as needed
    let options = NativeOptions::default();
    eframe::run_native(
        "Overclocking Assistant",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
