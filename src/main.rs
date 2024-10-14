// src/main.rs
mod orchestrator;
mod power;
mod tweaks;
mod utils;
mod widgets;

use std::collections::BTreeMap;

use eframe::{egui, App, Frame, NativeOptions};
use egui::{vec2, Button, FontId, RichText, Sense, Vec2};
use orchestrator::{TaskOrchestrator, TweakAction, TweakTask};
use power::{read_power_state, PowerState, SlowMode, SLOW_MODE_DESCRIPTION};
use tracing::Level;
use tweaks::{Tweak, TweakCategory, TweakId, TweakStatus};
use utils::{is_elevated, reboot_into_bios, reboot_system};
use widgets::{
    button::{action_button, ButtonState},
    switch::toggle_switch,
    TweakWidget,
};

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
const BUTTON_DIMENSIONS: Vec2 = vec2(40.0, 20.0);

/// Represents your application's main structure.
pub struct MyApp {
    pub tweaks: BTreeMap<TweakId, Tweak>,
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
        let app_span = tracing::span!(Level::INFO, "App Initialization");
        let _app_guard = app_span.enter();

        // Initialize tweaks
        let mut tweaks = tweaks::all();

        // Initialize the task orchestrator with the specified number of workers
        let orchestrator = TaskOrchestrator::new();

        // Initialize the current state of all tweaks
        for (id, tweak) in tweaks.iter_mut() {
            // Submit a task to read the initial state of each tweak
            let task = TweakTask {
                id: *id,
                method: tweak.method.clone(),
                action: TweakAction::ReadInitialState,
            };
            // Log an error if the task submission fails
            if let Err(e) = orchestrator.submit_task(task) {
                tracing::error!(
                    "Failed to submit initial state task for tweak {:?}: {:?}",
                    id,
                    e
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

    /// Count the tweaks that are pending a system reboot.
    fn count_tweaks_pending_reboot(&self) -> usize {
        self.tweaks
            .iter()
            .filter(|(_, tweak)| tweak.pending_reboot)
            .count()
    }

    /// Poll the orchestrator to check for any completed tasks and update tweak states.
    fn update_tweak_states(&mut self) {
        while let Some(result) = self.orchestrator.try_recv_result() {
            if let Some(tweak) = self.tweaks.get_mut(&result.id) {
                if result.success {
                    match tweak.get_status() {
                        TweakStatus::Applying => {
                            // Set status to Idle after successfully applying the tweak
                            tweak.set_status(TweakStatus::Idle);
                        }
                        TweakStatus::Idle => {
                            // If the initial state was read successfully, update the enabled state
                            if let Some(enabled) = result.enabled_state {
                                tweak.set_enabled(enabled);
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Handle error if the task did not succeed
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

    /// Draw the main UI grid with two columns for the tweaks.
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
                        // Draw each tweak category on the left side
                        self.draw_category_section(ui, category);
                    }
                });

                // **Second Column**
                ui.vertical(|ui| {
                    for category in TweakCategory::right() {
                        // Draw each tweak category on the right side
                        self.draw_category_section(ui, category);
                    }
                });
            });
    }

    /// Helper method to draw a single category section.
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

        // If no tweaks exist for the category, return early
        if category_tweaks.is_empty() {
            return;
        }

        // Render Category Header
        ui.heading(format!("{:?} Tweaks", category));
        ui.separator();
        ui.add_space(CONTAINER_VERTICAL_PADDING); // Use the constant for padding

        // Iterate over each tweak and draw using the tweak container
        for tweak_id in category_tweaks {
            self.draw_tweak_container(ui, tweak_id);
            ui.add_space(CONTAINER_VERTICAL_PADDING); // Use the constant for padding
        }

        ui.add_space(CONTAINER_VERTICAL_PADDING * 2.0); // Add extra vertical spacing between categories
    }

    /// Render the tweak container for a given tweak ID.
    fn draw_tweak_container(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        let desired_size = vec2(TWEAK_CONTAINER_WIDTH, TWEAK_CONTAINER_HEIGHT);
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());
        let mut child_ui = ui.child_ui(
            rect,
            *ui.layout(),
            Some(egui::UiStackInfo::new(egui::UiKind::Frame)),
        );

        // Draw a frame around each tweak container
        egui::Frame::group(child_ui.style())
            .fill(child_ui.visuals().faint_bg_color)
            .rounding(5.0)
            .inner_margin(egui::Margin::same(CONTAINER_INTERNAL_PADDING))
            .show(&mut child_ui, |ui| {
                // Use horizontal layout to arrange the tweak name and control widget
                ui.horizontal(|ui| {
                    if let Some(tweak) = self.tweaks.get(&tweak_id) {
                        // Render the tweak label
                        ui.label(
                            RichText::new(&tweak.name)
                                .text_style(egui::TextStyle::Body)
                                .strong(),
                        )
                        .on_hover_text(&tweak.description);

                        // Render the appropriate widget (toggle or button) based on tweak type
                        match tweak.widget {
                            TweakWidget::Toggle => self.draw_toggle_widget(ui, tweak_id),
                            TweakWidget::Button => self.draw_button_widget(ui, tweak_id),
                        }
                    }
                });
            });
    }

    /// Draw a toggle widget for a tweak, allowing users to enable or disable it.
    fn draw_toggle_widget(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        if let Some(tweak_entry) = self.tweaks.get_mut(&tweak_id) {
            let mut is_enabled = tweak_entry.is_enabled();
            let response_toggle = ui.add(toggle_switch(&mut is_enabled));

            // If the toggle was interacted with, update the tweak's status and submit a task
            if response_toggle.changed() {
                tweak_entry.set_enabled(is_enabled);
                tweak_entry.set_status(TweakStatus::Applying);
                let result = self.orchestrator.submit_task(TweakTask {
                    id: tweak_id,
                    method: tweak_entry.method.clone(),
                    action: if is_enabled {
                        TweakAction::Apply
                    } else {
                        TweakAction::Revert
                    },
                });
                // Handle any errors during task submission
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        tweak_entry.set_status(TweakStatus::Failed(e.to_string()));
                    }
                }
            }

            // If there was an error, display it in red text
            if let TweakStatus::Failed(ref err) = tweak_entry.get_status() {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
        }
    }

    /// Draw a button widget for a tweak, allowing users to apply the tweak.
    fn draw_button_widget(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        if let Some(tweak_entry) = self.tweaks.get_mut(&tweak_id) {
            // Determine the button state based on the tweak's status
            let button_state = match tweak_entry.get_status() {
                TweakStatus::Idle | TweakStatus::Failed(_) => ButtonState::Default,
                TweakStatus::Applying => ButtonState::InProgress,
                _ => ButtonState::Default,
            };

            let mut button_state_mut = button_state;
            let response_button = ui.add(action_button(&mut button_state_mut));

            // If the button was clicked and the tweak is idle, set the tweak status to applying
            if response_button.clicked() && button_state == ButtonState::Default {
                tweak_entry.set_status(TweakStatus::Applying);
                let result = self.orchestrator.submit_task(TweakTask {
                    id: tweak_id,
                    method: tweak_entry.method.clone(),
                    action: TweakAction::Apply,
                });
                // Handle any errors during task submission
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        tweak_entry.set_status(TweakStatus::Failed(e.to_string()));
                    }
                }
            }

            // If there was an error, display it in red text
            if let TweakStatus::Failed(ref err) = tweak_entry.get_status() {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
        }
    }

    /// Renders the status bar at the bottom with information and action buttons.
    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(STATUS_BAR_PADDING); // Apply the constant padding

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

                ui.separator(); // Vertical separator
                if ui
                    .add(Button::new("Restart Windows").min_size(BUTTON_DIMENSIONS))
                    .clicked()
                {
                    // Trigger system reboot and handle any errors
                    if let Err(e) = reboot_system() {
                        tracing::error!("Failed to initiate reboot: {:?}", e);
                        tinyfiledialogs::message_box_ok(
                            "Overclocking Assistant",
                            &format!("Failed to reboot the system: {:?}", e),
                            tinyfiledialogs::MessageBoxIcon::Error,
                        );
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
                                tracing::debug!("Rebooting into BIOS settings...");
                            }
                            Err(e) => {
                                tracing::error!("Failed to reboot into BIOS: {:?}", e);
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
                                    tracing::debug!("Slow mode enabled successfully.");
                                    self.slow_mode = true;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to enable slow mode: {:?}", e);
                                }
                            },
                            true => match self.disable_slow_mode() {
                                Ok(_) => {
                                    tracing::debug!("Slow mode disabled successfully.");
                                    self.slow_mode = false;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to disable slow mode: {:?}", e);
                                }
                            },
                        }
                    }
                });
            });

            ui.add_space(STATUS_BAR_PADDING); // Apply the constant padding
        });
    }
}

impl App for MyApp {
    /// The main update loop where the UI is rendered and interactions are handled.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Process any incoming results from the workers
        self.update_tweak_states();

        if !self.initial_states_loaded {
            // Display a loading screen while initial states are being read
            egui::CentralPanel::default().show(ctx, |ui| {
                // In the loading screen UI
                ui.vertical_centered(|ui| {
                    ui.add_space(200.0);
                    ui.heading("Reading system state...");
                    ui.add(egui::widgets::Spinner::new());
                });
            });
        } else {
            // Render the main UI once the initial state is loaded
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.draw_ui(ui);
                    });
            });

            // Render the status bar at the bottom
            self.draw_status_bar(ctx);
        }
    }

    /// Handles application exit by sending a shutdown message to the worker pool.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Submit a task to revert the low-res mode tweak when exiting
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
            tracing::error!("Failed to submit revert task for low-res mode: {:?}", e);
        }

        // Disable slow mode if it's enabled during application exit
        if let Err(e) = self.disable_slow_mode() {
            tracing::error!("Failed to disable slow mode during exit: {:?}", e);
        }
    }
}

fn main() -> eframe::Result<()> {
    // Check if the application is running with elevated privileges
    match is_elevated() {
        true => tracing::debug!("Running with elevated privileges."),
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

    // Set up window options
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]) // Adjusted size for better layout
            .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT / 2.0]), // Set a minimum size
        ..Default::default()
    };

    // Create a tracing span for the run_native call
    let run_span = tracing::span!(Level::INFO, "Run Native");
    run_span.in_scope(|| {
        tracing::trace!("Entering Run Native span.");
        eframe::run_native(
            "OC Tool",
            options,
            Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
        )
    })
}
