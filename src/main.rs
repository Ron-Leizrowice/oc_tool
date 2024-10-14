// src/main.rs
mod power;
mod tweaks;
mod utils;
mod widgets;
mod worker;

use std::collections::HashMap;

use eframe::{egui, App, Frame, NativeOptions};
use egui::{vec2, Button, FontId, RichText, Sense, Vec2};
use power::{read_power_state, PowerState, SlowMode, SLOW_MODE_DESCRIPTION};
use tinyfiledialogs::YesNo;
use tracing::{error, info, span, trace, Level};
use tweaks::{Tweak, TweakCategory, TweakId, TweakStatus};
use utils::{is_elevated, reboot_into_bios, reboot_system};
use widgets::{
    button::{action_button, ButtonState},
    switch::toggle_switch,
    TweakWidget,
};
use worker::{TaskOrchestrator, TweakAction, TweakTask};

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
const STATUS_BAR_PADDING: f32 = 5.0; // Padding inside the status bar

// Default Button Dimensions
const BUTTON_WIDTH: f32 = 40.0;
const BUTTON_HEIGHT: f32 = 20.0;

const BUTTON_DIMENSIONS: Vec2 = vec2(BUTTON_WIDTH, BUTTON_HEIGHT);

/// Represents your application's main structure.
pub struct MyApp {
    pub tweaks: HashMap<TweakId, Tweak>,
    pub orchestrator: TaskOrchestrator,

    // Power management fields
    pub power_state: PowerState,
    pub slow_mode: bool,

    // State tracking for initial state reads
    pub initial_states_loaded: bool,
    pub pending_initial_state_reads: usize,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize tracing spans for better context
        let app_span = span!(Level::INFO, "App Initialization");
        let _app_guard = app_span.enter();

        // Initialize tweaks
        let mut tweaks = tweaks::all();

        // Initialize the task orchestrator with the specified number of workers
        let orchestrator = TaskOrchestrator::new();

        // Initialize the current state of all tweaks
        for (id, tweak) in tweaks.iter_mut() {
            // Set initial status to Idle
            tweak.set_status(TweakStatus::Idle);
            // Submit a task to read the initial state
            let task = TweakTask {
                id: *id,
                method: tweak.method.clone(),
                action: TweakAction::ReadInitialState,
            };
            if let Err(e) = orchestrator.submit_task(task) {
                error!(
                    "Failed to submit initial state task for tweak {:?}: {:?}",
                    id, e
                );
            }
        }

        // Initialize the pending reads to the number of tweaks
        let pending_initial_state_reads = tweaks.len();

        Self {
            tweaks,
            orchestrator,
            power_state: read_power_state().unwrap(),
            slow_mode: false,
            initial_states_loaded: false,
            pending_initial_state_reads,
        }
    }

    fn count_tweaks_pending_reboot(&self) -> usize {
        self.tweaks
            .iter()
            .filter(|(_, tweak)| tweak.pending_reboot)
            .count()
    }

    /// Poll the orchestrator to check for any completed tasks.
    fn update_tweak_states(&mut self) {
        while let Some(result) = self.orchestrator.try_recv_result() {
            if let Some(tweak) = self.tweaks.get_mut(&result.id) {
                if result.success {
                    match tweak.get_status() {
                        TweakStatus::Applying => {
                            // Set status to Idle after applying
                            tweak.set_status(TweakStatus::Idle);
                        }
                        TweakStatus::Idle => {
                            // For initial state read, set enabled state
                            if let Some(enabled) = result.enabled_state {
                                tweak.set_enabled(enabled);
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Handle error
                    tweak.set_status(TweakStatus::Failed(
                        result.error.unwrap_or_else(|| "Unknown error".to_string()),
                    ));
                }
            }

            // Decrement the counter if the action was ReadInitialState
            if let TweakAction::ReadInitialState = result.action {
                self.pending_initial_state_reads =
                    self.pending_initial_state_reads.saturating_sub(1);

                // If all initial states are read, set initial_states_loaded to true
                if self.pending_initial_state_reads == 0 {
                    self.initial_states_loaded = true;
                }
            }
        }
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) {
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
    fn draw_category_section(&mut self, ui: &mut egui::Ui, category: TweakCategory) {
        // Filter tweaks belonging to the current category
        let category_tweaks: Vec<TweakId> = self
            .tweaks
            .iter()
            .filter_map(|tweak_entry| {
                let (id, tweak) = tweak_entry;
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

    fn draw_tweak_container(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
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
                        if let Some(tweak_entry) = self.tweaks.get(&tweak_id) {
                            ui.label(
                                egui::RichText::new(&tweak_entry.name)
                                    .text_style(egui::TextStyle::Body)
                                    .strong(),
                            )
                            .on_hover_text(&tweak_entry.description);
                        }
                    });

                    // **Widget Section**
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Acquire the lock briefly to get the tweak status and enabled state
                        if let Some(tweak_entry) = self.tweaks.get(&tweak_id) {
                            let tweak_status = tweak_entry.get_status();
                            let is_enabled = tweak_entry.is_enabled();
                            let widget_type = tweak_entry.widget.clone();


                            match widget_type {
                                TweakWidget::Toggle => {
                                    // Toggle Switch Widget
                                    let mut is_enabled_mut = is_enabled;
                                    let response_toggle =
                                        ui.add(toggle_switch(&mut is_enabled_mut));

                                    // Handle toggle interaction
                                    if response_toggle.changed() {
                                        // Acquire mutable reference to tweak to set status
                                        if let Some(tweak) = self.tweaks.get_mut(&tweak_id) {
                                            // Update the tweak's enabled state
                                            tweak.set_enabled(is_enabled_mut);
                                            // Set the status to Applying
                                            tweak.set_status(TweakStatus::Applying);
                                        }
                                        // Dispatch the apply or revert task based on the new state
                                        let action = if is_enabled_mut {
                                            TweakAction::Apply
                                        } else {
                                            TweakAction::Revert
                                        };
                                        if let Some(tweak_entry) = self.tweaks.get(&tweak_id) {
                                            let task = TweakTask {
                                                id: tweak_id,
                                                method: tweak_entry.method.clone(),
                                                action,
                                            };
                                            if let Err(e) = self.orchestrator.submit_task(task) {
                                                error!(
                                                    "Failed to submit task for tweak {:?}: {:?}",
                                                    tweak_id, e
                                                );
                                            }
                                        }
                                    }

                                    // Handle error messages
                                    if let TweakStatus::Failed(ref err) = tweak_status {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("Error: {}", err),
                                        );
                                    }
                                }
                                TweakWidget::Button => {
                                    // Determine button state based on tweak_status
                                    let button_state = match tweak_status {
                                        TweakStatus::Idle | TweakStatus::Failed(_) => {
                                            ButtonState::Default
                                        }
                                        TweakStatus::Applying => ButtonState::InProgress,
                                        _ => ButtonState::Default,
                                    };

                                    // Create a mutable variable for the button state (since the widget expects &mut)
                                    let mut button_state_mut = button_state;

                                    // Add the action button widget
                                    let response_button =
                                        ui.add(action_button(&mut button_state_mut));

                                    // Handle button click
                                    if response_button.clicked() && button_state == ButtonState::Default {
                                        // Acquire mutable reference to the tweak to set status to Applying
                                        if let Some(tweak) = self.tweaks.get_mut(&tweak_id) {
                                            tweak.set_status(TweakStatus::Applying);
                                        }
                                        // Dispatch the apply task
                                        if let Some(tweak_entry) = self.tweaks.get(&tweak_id) {
                                            let task = TweakTask {
                                                id: tweak_id,
                                                method: tweak_entry.method.clone(),
                                                action: TweakAction::Apply,
                                            };
                                            if let Err(e) = self.orchestrator.submit_task(task) {
                                                error!("Failed to submit apply task for tweak {:?}: {:?}", tweak_id, e);
                                            }
                                        }
                                    }

                                    // Handle error messages
                                    if let TweakStatus::Failed(ref err) = tweak_status {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("Error: {}", err),
                                        );
                                    }
                                }
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
                ui.label(RichText::new("v0.1.0a").font(FontId::proportional(16.0)));
                ui.separator(); // Vertical separator

                // Tweaks pending restart
                let pending_reboot_count = self.count_tweaks_pending_reboot();

                ui.label(
                    RichText::new(format!(
                        "{} tweak{} pending restart",
                        pending_reboot_count,
                        if pending_reboot_count != 1 { "s" } else { "" }
                    ))
                    .font(FontId::proportional(14.0)),
                );

                // If there are pending reboots, show a reboot button
                if pending_reboot_count > 0 {
                    ui.separator(); // Vertical separator
                    if ui
                        .add(Button::new("Restart Windows").min_size(BUTTON_DIMENSIONS))
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
                        .add(Button::new("Reboot into BIOS").min_size(BUTTON_DIMENSIONS))
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
                        .add(Button::new(slow_mode_label).min_size(BUTTON_DIMENSIONS))
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
        self.update_tweak_states();

        if !self.initial_states_loaded {
            // Display a loading screen
            egui::CentralPanel::default().show(ctx, |ui| {
                // In the loading screen UI
                ui.vertical_centered(|ui| {
                    ui.add_space(200.0);
                    ui.heading("Reading system state...");
                    ui.add(egui::widgets::Spinner::new());
                });
            });
        } else {
            // Render the main UI
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.draw_ui(ui);
                    });
            });

            // Render the status bar
            self.draw_status_bar(ctx);
        }
    }

    /// Handles application exit by sending a shutdown message to the worker pool.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let task = TweakTask {
            id: TweakId::LowResMode,
            method: self
                .tweaks
                .get(&TweakId::LowResMode)
                .unwrap()
                .method
                .clone(),
            action: TweakAction::Revert,
        };
        if let Err(e) = self.orchestrator.submit_task(task) {
            error!("Failed to submit revert task for low-res mode: {:?}", e);
        }

        // Disable slow mode
        if let Err(e) = self.disable_slow_mode() {
            error!("Failed to disable slow mode during exit: {:?}", e);
        }
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
