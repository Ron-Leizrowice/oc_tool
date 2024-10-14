#![windows_subsystem = "windows"]

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
const WINDOW_HEIGHT: f32 = 900.0;

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
        let app_span = tracing::span!(Level::INFO, "App Initialization");
        let _app_guard = app_span.enter();

        let mut tweaks = tweaks::all();
        let orchestrator = TaskOrchestrator::new();

        for (id, tweak) in tweaks.iter_mut() {
            let task = TweakTask {
                id: *id,
                method: tweak.method.clone(),
                action: TweakAction::ReadInitialState,
            };
            if let Err(e) = orchestrator.submit_task(task) {
                tracing::error!(
                    "Failed to submit initial state task for tweak {:?}: {:?}",
                    id,
                    e
                );
            }
        }

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

    fn update_tweak_states(&mut self) {
        while let Some(result) = self.orchestrator.try_recv_result() {
            if let Some(tweak) = self.tweaks.get_mut(&result.id) {
                if result.success {
                    match tweak.status {
                        TweakStatus::Applying => {
                            tweak.status = TweakStatus::Idle;
                        }
                        TweakStatus::Idle => {
                            if let Some(enabled) = result.enabled_state {
                                tweak.enabled = enabled;
                            }
                        }
                        _ => {}
                    }
                } else {
                    tweak.status = TweakStatus::Failed(
                        result.error.unwrap_or_else(|| "Unknown error".to_string()),
                    );
                }
            }

            if let TweakAction::ReadInitialState = result.action {
                self.pending_initial_state_reads =
                    self.pending_initial_state_reads.saturating_sub(1);

                if self.pending_initial_state_reads == 0 {
                    self.initial_states_loaded = true;
                }
            }
        }
    }

    fn cleanup(&mut self) {
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

        if let Err(e) = self.disable_slow_mode() {
            tracing::error!("Failed to disable slow mode during exit: {:?}", e);
        }
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("main_columns_grid")
            .num_columns(2)
            .spacing(egui::vec2(GRID_HORIZONTAL_SPACING, 0.0))
            .striped(true)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for category in TweakCategory::left() {
                        self.draw_category_section(ui, category);
                    }
                });

                ui.vertical(|ui| {
                    for category in TweakCategory::right() {
                        self.draw_category_section(ui, category);
                    }
                });
            });
    }

    fn draw_category_section(&mut self, ui: &mut egui::Ui, category: TweakCategory) {
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

        ui.heading(format!("{:?} Tweaks", category));
        ui.separator();
        ui.add_space(CONTAINER_VERTICAL_PADDING);

        for tweak_id in category_tweaks {
            self.draw_tweak_container(ui, tweak_id);
            ui.add_space(CONTAINER_VERTICAL_PADDING);
        }

        ui.add_space(CONTAINER_VERTICAL_PADDING * 2.0);
    }

    fn draw_tweak_container(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        let desired_size = vec2(TWEAK_CONTAINER_WIDTH, TWEAK_CONTAINER_HEIGHT);
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());
        let mut child_ui = ui.child_ui(
            rect,
            egui::Layout::default().with_main_wrap(false),
            Some(egui::UiStackInfo::new(egui::UiKind::Frame)),
        );

        // First, get an immutable reference to the tweak data
        let tweak_data = self.tweaks.get(&tweak_id).map(|tweak| {
            (
                tweak.name.clone(),
                tweak.description.clone(),
                tweak.widget.clone(),
            )
        });

        if let Some((tweak_name, tweak_description, tweak_widget)) = tweak_data {
            // Now use the tweak data to create the UI
            egui::Frame::group(child_ui.style())
                .fill(child_ui.visuals().faint_bg_color)
                .rounding(5.0)
                .inner_margin(egui::Margin::same(CONTAINER_INTERNAL_PADDING))
                .show(&mut child_ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.set_width(TWEAK_CONTAINER_WIDTH - 2.0 * CONTAINER_INTERNAL_PADDING);

                        // Draw tweak name and description
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(&tweak_name)
                                    .text_style(egui::TextStyle::Body)
                                    .strong(),
                            )
                            .on_hover_text(&tweak_description);
                        });

                        // Draw the appropriate widget using a mutable reference to `self`
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            match tweak_widget {
                                TweakWidget::Toggle => self.draw_toggle_widget(ui, tweak_id),
                                TweakWidget::Button => self.draw_button_widget(ui, tweak_id),
                            }
                        });
                    });
                });
        }
    }

    fn draw_toggle_widget(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        if let Some(tweak_entry) = self.tweaks.get_mut(&tweak_id) {
            let mut is_enabled = tweak_entry.enabled;
            let response_toggle = ui.add(toggle_switch(&mut is_enabled));

            if response_toggle.changed() {
                tweak_entry.enabled = is_enabled;
                if tweak_entry.requires_reboot {
                    tweak_entry.pending_reboot = true;
                }
                tweak_entry.status = TweakStatus::Applying;
                let result = self.orchestrator.submit_task(TweakTask {
                    id: tweak_id,
                    method: tweak_entry.method.clone(),
                    action: if is_enabled {
                        TweakAction::Apply
                    } else {
                        TweakAction::Revert
                    },
                });
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        tweak_entry.status = TweakStatus::Failed(e.to_string());
                    }
                }
            }

            if let TweakStatus::Failed(ref err) = tweak_entry.status {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
        }
    }

    fn draw_button_widget(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        if let Some(tweak_entry) = self.tweaks.get_mut(&tweak_id) {
            let button_state = match tweak_entry.status {
                TweakStatus::Idle | TweakStatus::Failed(_) => ButtonState::Default,
                TweakStatus::Applying => ButtonState::InProgress,
                _ => ButtonState::Default,
            };

            let mut button_state_mut = button_state;
            let response_button = ui.add(action_button(&mut button_state_mut));

            if response_button.clicked() && button_state == ButtonState::Default {
                tweak_entry.status = TweakStatus::Applying;
                let result = self.orchestrator.submit_task(TweakTask {
                    id: tweak_id,
                    method: tweak_entry.method.clone(),
                    action: TweakAction::Apply,
                });
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        tweak_entry.status = TweakStatus::Failed(e.to_string());
                    }
                }
            }

            if let TweakStatus::Failed(ref err) = tweak_entry.status {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
        }
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(STATUS_BAR_PADDING);

            ui.horizontal(|ui| {
                ui.label(RichText::new("v0.1.5a").font(FontId::proportional(16.0)));
                ui.separator();

                let pending_reboot_count = self.count_tweaks_pending_reboot();

                ui.label(
                    RichText::new(format!(
                        "{} tweak{} pending restart",
                        pending_reboot_count,
                        if pending_reboot_count != 1 { "s" } else { "" }
                    ))
                    .font(FontId::proportional(14.0)),
                );

                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(Button::new("Reboot into BIOS").min_size(BUTTON_DIMENSIONS))
                        .clicked()
                    {
                        match reboot_into_bios() {
                            Ok(_) => {
                                tracing::debug!("Rebooting into BIOS settings...");
                                self.cleanup();
                            }
                            Err(e) => {
                                tracing::error!("Failed to reboot into BIOS: {:?}", e);
                            }
                        }
                    }

                    if ui
                        .add(Button::new("Restart Windows").min_size(BUTTON_DIMENSIONS))
                        .clicked()
                    {
                        self.cleanup();
                        if let Err(e) = reboot_system() {
                            tracing::error!("Failed to initiate reboot: {:?}", e);
                            tinyfiledialogs::message_box_ok(
                                "Overclocking Assistant",
                                &format!("Failed to reboot the system: {:?}", e),
                                tinyfiledialogs::MessageBoxIcon::Error,
                            );
                        }
                    }

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

            ui.add_space(STATUS_BAR_PADDING);
        });
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.update_tweak_states();

        if !self.initial_states_loaded {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(200.0);
                    ui.heading("Reading system state...");
                    ui.add(egui::widgets::Spinner::new());
                });
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.draw_ui(ui);
                    });
            });

            self.draw_status_bar(ctx);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cleanup();
    }
}

fn main() -> eframe::Result<()> {
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

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT / 2.0]),
        ..Default::default()
    };

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
