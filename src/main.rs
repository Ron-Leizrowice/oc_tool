// src/main.rs
mod power;
mod tweaks;
mod utils;
mod widgets;
mod worker;

use std::sync::Arc;

use dashmap::DashMap;
use eframe::{egui, App, Frame, NativeOptions};
use egui::{vec2, Button, FontId, RichText, Sense};
use power::{read_power_state, PowerState, SlowMode, SLOW_MODE_DESCRIPTION};
use tinyfiledialogs::YesNo;
use tracing::{error, info, span, trace, warn, Level};
use tweaks::{Tweak, TweakCategory, TweakId, TweakStatus};
use utils::{is_elevated, reboot_into_bios, reboot_system};
use widgets::{
    button::{action_button, ButtonState},
    switch::toggle_switch,
    TweakWidget,
};

use crate::worker::{WorkerPool, WorkerResult, WorkerTask};

// Constants for layout and spacing
const WINDOW_WIDTH: f32 = TWEAK_CONTAINER_WIDTH * 2.0 + GRID_HORIZONTAL_SPACING * 2.0 + 10.0;
const WINDOW_HEIGHT: f32 = 840.0;

// Controls the dimensions of each tweak container.
const TWEAK_CONTAINER_HEIGHT: f32 = 30.0;
const TWEAK_CONTAINER_WIDTH: f32 = 280.0;

/// Controls the padding for tweak containers.
const CONTAINER_VERTICAL_PADDING: f32 = 0.0;
const CONTAINER_INTERNAL_PADDING: f32 = 4.0;

// Horizontal spacing between grid columns
const GRID_HORIZONTAL_SPACING: f32 = 20.0; // Space between grid columns

// Status Bar Padding
const STATUS_BAR_PADDING: f32 = 10.0; // Padding inside the status bar

const NUM_WORKERS: usize = 8;

/// Represents your application's main structure.
pub struct MyApp {
    pub tweaks: Arc<DashMap<TweakId, Tweak>>,
    pub worker_pool: WorkerPool,

    // Power management fields
    pub power_state: PowerState,
    pub slow_mode: bool,
}

impl MyApp {
    /// Initializes the application by setting up tweaks and the worker pool.
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize tracing spans for better context
        let app_span = span!(Level::INFO, "App Initialization");
        let _app_guard = app_span.enter();

        // Initialize tweaks
        let tweaks = Arc::new(tweaks::all());

        // Initialize worker pool
        let worker_pool = WorkerPool::new(NUM_WORKERS, Arc::clone(&tweaks));

        // Initialize the current state of all tweaks
        for tweak_id in tweaks.iter().map(|entry| *entry.key()) {
            worker_pool
                .send_task(WorkerTask::ReadInitialState { id: tweak_id })
                .expect("Failed to send ReadInitialState task to worker pool");
        }

        Self {
            tweaks,
            worker_pool,
            power_state: read_power_state().unwrap(),
            slow_mode: false,
        }
    }

    fn count_tweaks_pending_reboot(&self) -> usize {
        self.tweaks
            .iter()
            .filter(|tweak| tweak.value().pending_reboot)
            .count()
    }

    /// Processes any incoming results from the workers and updates tweak statuses.
    fn process_worker_results(&mut self) {
        while let Some(result) = self.worker_pool.try_recv_result() {
            match result {
                WorkerResult::TweakApplied { id, success, error } => {
                    if let Some(mut tweak) = self.tweaks.get_mut(&id) {
                        if success {
                            tweak.set_status(TweakStatus::Idle);

                            if tweak.requires_reboot {
                                tweak.set_pending_reboot(true);
                                info!("{id:?} -> Tweak applied successfully. Pending reboot.");
                            } else {
                                info!("{id:?} -> Tweak applied successfully. No reboot required.");
                            }
                        } else {
                            tweak.set_status(TweakStatus::Failed(
                                error.unwrap_or_else(|| "Unknown error".to_string()),
                            ));
                            warn!("{id:?} -> Tweak apply failed: {:?}", tweak.get_status());
                        }
                    } else {
                        warn!("Received result for unknown tweak: {id:?}");
                    }
                }
                WorkerResult::TweakReverted { id, success, error } => {
                    if let Some(mut tweak) = self.tweaks.get_mut(&id) {
                        if success {
                            tweak.set_status(TweakStatus::Idle);

                            if tweak.requires_reboot {
                                tweak.set_pending_reboot(false);
                                info!(
                                    "{id:?} -> Tweak reverted successfully. Pending reboot cleared.",
                                );
                            } else {
                                info!("{id:?} -> Tweak reverted successfully. No reboot required.",);
                            }
                        } else {
                            tweak.set_status(TweakStatus::Failed(
                                error.unwrap_or_else(|| "Unknown error".to_string()),
                            ));
                            warn!("{id:?} -> Tweak reversion failed: {:?}", tweak.get_status());
                        }
                    } else {
                        warn!("Received result for unknown tweak: {id:?}");
                    }
                }
                WorkerResult::InitialStateRead { id, success, error } => {
                    if let Some(mut tweak) = self.tweaks.get_mut(&id) {
                        if success {
                            tweak.set_status(TweakStatus::Idle);
                            info!(
                                "{id:?} -> Initial state read successfully. Setting tweak to Idle."
                            );
                        } else {
                            tweak.set_status(TweakStatus::Failed(
                                error.unwrap_or_else(|| "Unknown error".to_string()),
                            ));
                            warn!(
                                "{id:?} -> Failed to read initial state: {:?}",
                                tweak.get_status()
                            );
                        }
                    } else {
                        warn!("Received InitialStateRead result for unknown tweak: {id:?}");
                    }
                }
                WorkerResult::ShutdownComplete => {
                    info!("Worker has shut down gracefully.");
                }
            }
        }
    }

    /// Dispatches a task to apply or revert a tweak based on user interaction.
    fn dispatch_task(&self, id: TweakId, apply: bool) {
        let task = if apply {
            WorkerTask::ApplyTweak { id }
        } else {
            WorkerTask::RevertTweak { id }
        };

        if let Err(e) = self.worker_pool.send_task(task) {
            error!("Failed to send task to worker pool: {}", e);
        }
    }

    fn draw_ui(&self, ui: &mut egui::Ui) {
        // Create a two-column grid with custom spacing
        egui::Grid::new("main_columns_grid")
            .num_columns(2)
            .spacing(egui::vec2(GRID_HORIZONTAL_SPACING, 0.0))
            .striped(true)
            .show(ui, |ui| {
                // **First Column**
                ui.vertical(|ui| {
                    for category in TweakCategory::left() {
                        self.draw_category_section(ui, category);
                    }
                });

                // **Second Column**
                ui.vertical(|ui| {
                    for category in TweakCategory::right() {
                        self.draw_category_section(ui, category);
                    }
                });
            });
    }

    /// Helper method to draw a single category section
    fn draw_category_section(&self, ui: &mut egui::Ui, category: TweakCategory) {
        // Filter tweaks belonging to the current category
        let category_tweaks: Vec<TweakId> = self
            .tweaks
            .iter()
            .filter_map(|tweak_entry| {
                let (id, tweak) = tweak_entry.pair();
                if tweak.category == category {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        if category_tweaks.is_empty() {
            return;
        }

        // Render Category Header
        ui.heading(format!("{:?} Tweaks", category));
        ui.separator();
        ui.add_space(CONTAINER_VERTICAL_PADDING); // Use the constant

        // Iterate over each tweak and draw using the tweak container
        for tweak_id in category_tweaks {
            self.draw_tweak_container(ui, tweak_id);
            ui.add_space(CONTAINER_VERTICAL_PADDING); // Use the constant
        }

        ui.add_space(CONTAINER_VERTICAL_PADDING * 2.0); // Add some vertical spacing between categories
    }

    fn draw_tweak_container(&self, ui: &mut egui::Ui, tweak_id: TweakId) {
        // Define the desired size for the tweak container
        let desired_size = egui::vec2(TWEAK_CONTAINER_WIDTH, TWEAK_CONTAINER_HEIGHT);

        // Allocate a fixed-size rectangular area for the container
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());

        // Create a child UI within the allocated rect
        let mut child_ui = ui.child_ui(
            rect,
            *ui.layout(),
            Some(egui::UiStackInfo::new(egui::UiKind::Frame)),
        );

        // Render the Frame within the child UI
        egui::Frame::group(child_ui.style())
            .fill(child_ui.visuals().faint_bg_color)
            .rounding(5.0)
            .inner_margin(egui::Margin::same(CONTAINER_INTERNAL_PADDING))
            .show(&mut child_ui, |ui| {
                // Use horizontal layout to align label left and widget right
                ui.horizontal(|ui| {
                    // **Label Section**
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        if let Some(tweak) = self.tweaks.get(&tweak_id) {
                            ui.label(
                                egui::RichText::new(&tweak.name)
                                    .text_style(egui::TextStyle::Body)
                                    .strong(),
                            )
                            .on_hover_text(&tweak.description);
                        }
                    });

                    // **Widget Section**
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(mut tweak) = self.tweaks.get_mut(&tweak_id) {
                            match tweak.widget {
                                TweakWidget::ToggleSwitch => {
                                    // Toggle Switch Widget
                                    let mut is_enabled = tweak.is_enabled();
                                    let response_toggle = ui.add(toggle_switch(&mut is_enabled));

                                    // Handle toggle interaction
                                    if response_toggle.changed() {
                                        // Update the enabled state
                                        tweak.set_enabled(is_enabled);

                                        // Dispatch the apply or revert task based on the new state
                                        self.dispatch_task(tweak_id, is_enabled);
                                    }

                                    // Handle error messages
                                    if let TweakStatus::Failed(ref err) = tweak.get_status() {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("Error: {}", err),
                                        );
                                    }
                                }
                                TweakWidget::ActionButton => {
                                    // Apply Button Widget
                                    let button_state = match tweak.get_status() {
                                        TweakStatus::Idle => ButtonState::Default,
                                        TweakStatus::Applying => ButtonState::InProgress,
                                        TweakStatus::Failed(_) => ButtonState::Default, // Reset to Default on failure
                                    };

                                    // Create a mutable reference to the button state
                                    let mut button_state_mut = button_state;

                                    // Add the ApplyButton widget
                                    let response_button =
                                        ui.add(action_button(&mut button_state_mut));

                                    // If the button was clicked and is not in progress
                                    if response_button.clicked()
                                        && tweak.get_status() != TweakStatus::Applying
                                    {
                                        // Update the tweak's status to Applying
                                        tweak.set_status(TweakStatus::Applying);

                                        // Dispatch the apply task
                                        self.dispatch_task(tweak_id, true);
                                    }

                                    // Handle error messages
                                    if let TweakStatus::Failed(ref err) = tweak.get_status() {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("Error: {}", err),
                                        );
                                    }
                                } // Handle other widget types here
                            }
                        }
                    });
                });
            });
    }

    /// Renders the status bar at the bottom with divisions.
    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(STATUS_BAR_PADDING); // Apply the constant

            ui.horizontal(|ui| {
                // **Version Label**
                ui.label(RichText::new("v1.0.1").font(FontId::proportional(16.0)));
                ui.separator(); // Vertical separator

                // Tweaks pending restart
                let pending_reboot_count = self.count_tweaks_pending_reboot();

                ui.label(
                    RichText::new(format!(
                        "{} tweak{} pending restart",
                        pending_reboot_count,
                        if pending_reboot_count != 1 { "s" } else { "" }
                    ))
                    .font(FontId::proportional(16.0)),
                );

                // If there are pending reboots, show a reboot button
                if pending_reboot_count > 0 {
                    ui.separator(); // Vertical separator
                    if ui
                        .add(Button::new("Restart Windows").min_size(vec2(40.0, 30.0)))
                        .clicked()
                    {
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
                // Right-aligned status bar items
                ui.separator(); // Vertical separator
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // **Reboot into BIOS Button**
                    if ui
                        .add(Button::new("Reboot into BIOS").min_size(vec2(40.0, 30.0)))
                        .clicked()
                    {
                        match reboot_into_bios() {
                            Ok(_) => {
                                info!("Rebooting into BIOS settings...");
                            }
                            Err(e) => {
                                error!("Failed to reboot into BIOS: {:?}", e);
                            }
                        }
                    }

                    // **Slow Mode Toggle Button**
                    let slow_mode_label = match self.slow_mode {
                        true => "Slow Mode: ON",
                        false => "Slow Mode: OFF",
                    };

                    if ui
                        .add(Button::new(slow_mode_label).min_size(vec2(40.0, 30.0)))
                        .on_hover_text(SLOW_MODE_DESCRIPTION)
                        .clicked()
                    {
                        match self.slow_mode {
                            false => match self.enable_slow_mode() {
                                Ok(_) => {
                                    info!("Slow mode enabled successfully.");
                                    self.slow_mode = true;
                                }
                                Err(e) => {
                                    error!("Failed to enable slow mode: {:?}", e);
                                }
                            },
                            true => match self.disable_slow_mode() {
                                Ok(_) => {
                                    info!("Slow mode disabled successfully.");
                                    self.slow_mode = false;
                                }
                                Err(e) => {
                                    error!("Failed to disable slow mode: {:?}", e);
                                }
                            },
                        }
                    }
                });
            });

            ui.add_space(STATUS_BAR_PADDING); // Apply the constant
        });
    }
}

impl App for MyApp {
    /// The main update loop where the UI is rendered and interactions are handled.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Process any incoming results from the workers
        self.process_worker_results();

        // Render the main UI with a vertical scrollbar
        egui::CentralPanel::default().show(ctx, |ui| {
            // Start a vertical scroll area that takes all available space
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2]) // Prevent the scroll area from shrinking
                .show(ui, |ui| {
                    // Call your existing UI drawing function
                    self.draw_ui(ui);
                });
        });

        // Render the status bar
        self.draw_status_bar(ctx);
    }

    /// Handles application exit by sending a shutdown message to the worker pool.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Disable low res mode
        self.dispatch_task(TweakId::LowResMode, false);
        // disable slow mode
        self.disable_slow_mode().unwrap();

        // close the worker pool
        self.worker_pool.shutdown(NUM_WORKERS);
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
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]) // Adjusted size for better layout
            .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT / 2.0]), // Set a minimum size
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
