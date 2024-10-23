// src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::collections::BTreeMap;

use eframe::{egui, App, Frame, NativeOptions};
use egui::{Button, FontId, RichText};
use egui_dialogs::{DialogDetails, Dialogs, StandardDialog, StandardReply};
use oc_tool::{
    constants::{
        LABEL_FONT_SIZE, TWEAK_CONTAINER_HEIGHT, TWEAK_CONTAINER_WIDTH, UI_PADDING, UI_SPACING,
        WINDOW_HEIGHT, WINDOW_WIDTH,
    },
    orchestrator::{TaskOrchestrator, TweakAction, TweakTask},
    tweaks::{self, Tweak, TweakCategory, TweakId, TweakStatus},
    ui::{
        button::{ActionButton, ButtonState, BUTTON_DIMENSIONS},
        switch::ToggleSwitch,
        TweakWidget,
    },
    utils::{
        windows::{is_elevated, reboot_into_bios, reboot_system},
        winring0::{setup_winring0_driver, WINRING0_DRIVER},
    },
};
use tracing::Level;
use tracing_subscriber::{self};

/// Represents your application's main structure.
pub struct MyApp {
    pub tweaks: BTreeMap<TweakId, Tweak<'static>>,
    pub orchestrator: TaskOrchestrator,

    // State tracking for initial state reads
    pub initial_states_loaded: bool,
    pub pending_initial_state_reads: usize,

    pub dialogs: Dialogs<'static>,

    selected_category: Option<TweakCategory>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let app_span = tracing::span!(Level::INFO, "App Initialization");
        let _app_guard = app_span.enter();

        let mut tweaks = tweaks::all_tweaks();
        let orchestrator = TaskOrchestrator::new();

        let pending_initial_state_reads = tweaks.len();

        let mut dialogs = Dialogs::new();

        // Check for elevated privileges
        if !is_elevated() {
            dialogs.add(DialogDetails::new(
                StandardDialog::info("Warning", "This program must be run in administrator mode.")
                    .buttons(vec![("OK".into(), StandardReply::Ok)]),
            ));
        } else {
            // Attempt to verify WinRing0 setup
            match setup_winring0_driver() {
                Ok(_) => {
                    tracing::debug!("WinRing0 driver setup successful.");
                }
                Err(err) => {
                    tracing::debug!("WinRing0 driver setup failed: {:?}", err);

                    // Add a dialog with the list of reasons
                    dialogs.add(DialogDetails::new(
                        StandardDialog::error(
                            "WinRing0 Initialization Failed",
                            format!("Failed to initialize WinRing0:\n{}", err),
                        )
                        .buttons(vec![("OK".into(), StandardReply::Ok)]),
                    ));
                }
            }
        }

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
                // Add a dialog to inform the user about the task submission failure
                dialogs.add(DialogDetails::new(
                    StandardDialog::error(
                        "Initialization Error",
                        format!("Failed to initialize tweak {:?}: {:?}", id, e),
                    )
                    .buttons(vec![("OK".into(), StandardReply::Ok)]),
                ));
            }
        }

        Self {
            tweaks,
            orchestrator,
            initial_states_loaded: false,
            pending_initial_state_reads,
            dialogs,
            selected_category: Some(TweakCategory::System),
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
                    // Set the tweak status to Failed with the error
                    let error_message = result.error.unwrap();
                    tracing::error!("Failed to process tweak {:?}: {}", result.id, error_message);
                    tweak.status = TweakStatus::Failed(error_message.to_string());

                    // Add a dialog to inform the user about the tweak failure
                    self.dialogs.add(DialogDetails::new(
                        StandardDialog::error(
                            "Tweak Application Error",
                            format!("Failed to apply tweak {:?}: {}", result.id, error_message),
                        )
                        .buttons(vec![("OK".into(), StandardReply::Ok)]),
                    ));
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

        // slow_mode and ultimate_performance are mutually exclusive
        let slow_mode_state = self
            .tweaks
            .get(&TweakId::SlowMode)
            .map(|t| t.enabled)
            .unwrap_or(false);
        let ultimate_performance_state = self
            .tweaks
            .get(&TweakId::UltimatePerformancePlan)
            .map(|t| t.enabled)
            .unwrap_or(false);

        if slow_mode_state && ultimate_performance_state {
            // Instead of panicking, add an error dialog
            tracing::error!("Both Slow Mode and Ultimate Performance are enabled simultaneously.");
            self.dialogs.add(DialogDetails::new(
                StandardDialog::error(
                    "Configuration Conflict",
                    "Both Slow Mode and Ultimate Performance are enabled at the same time. Please disable one to proceed.",
                )
                .buttons(vec![("OK".into(), StandardReply::Ok)]),
            ));
            // Optionally, you can automatically disable one to resolve the conflict
            // For example, disable Ultimate Performance if Slow Mode is enabled
            if let Some(tweak) = self.tweaks.get_mut(&TweakId::UltimatePerformancePlan) {
                tweak.enabled = false;
            }
        } else if slow_mode_state {
            if let Some(tweak) = self.tweaks.get_mut(&TweakId::UltimatePerformancePlan) {
                tweak.enabled = false;
            }
        } else if ultimate_performance_state {
            if let Some(tweak) = self.tweaks.get_mut(&TweakId::SlowMode) {
                tweak.enabled = false;
            }
        }
    }

    fn cleanup(&mut self) {
        // disable low-res mode
        if let Err(e) = self.orchestrator.submit_task(TweakTask {
            id: TweakId::LowResMode,
            method: self
                .tweaks
                .get(&TweakId::LowResMode)
                .unwrap()
                .method
                .clone(),
            action: TweakAction::Revert,
        }) {
            tracing::error!("Failed to submit revert task for low-res mode: {:?}", e);
        }
        // disable slow-mode
        if let Err(e) = self.orchestrator.submit_task(TweakTask {
            id: TweakId::SlowMode,
            method: self.tweaks.get(&TweakId::SlowMode).unwrap().method.clone(),
            action: TweakAction::Revert,
        }) {
            tracing::error!("Failed to submit revert task for slow mode: {:?}", e);
        }
        // drop the WinRing0 driver
        drop(WINRING0_DRIVER.lock().unwrap());
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.columns(3, |columns| {
                for category in TweakCategory::left() {
                    self.draw_category_section(&mut columns[0], category);
                }
                for category in TweakCategory::middle() {
                    self.draw_category_section(&mut columns[1], category);
                }
                for category in TweakCategory::right() {
                    self.draw_category_section(&mut columns[2], category);
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

        for tweak_id in category_tweaks {
            // Wrap each tweak container in a fixed width Frame to maintain consistency
            egui::Frame::none()
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE)
                .show(ui, |ui| {
                    ui.set_width(TWEAK_CONTAINER_WIDTH);
                    self.draw_tweak_container(ui, tweak_id);
                });
        }

        ui.add_space(UI_SPACING);
    }

    fn draw_tweak_container(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        // Retrieve tweak data
        let tweak_data = self
            .tweaks
            .get(&tweak_id)
            .map(|tweak| (tweak.name, tweak.description, tweak.widget));

        if let Some((tweak_name, tweak_description, tweak_widget)) = tweak_data {
            // Define a unique grid ID for each tweak to avoid conflicts
            egui::Grid::new(format!("tweak_grid_{:?}", tweak_id))
                .num_columns(2)
                .striped(false)
                .min_col_width(TWEAK_CONTAINER_WIDTH - BUTTON_DIMENSIONS[0] - UI_SPACING * 2.0)
                .spacing([UI_SPACING, 0.0])
                .show(ui, |ui| {
                    // **First Column**: Collapsing Header with Description
                    ui.vertical(|ui| {
                        ui.collapsing(tweak_name, |ui| {
                            ui.vertical(|ui| {
                                ui.label(tweak_description);
                                // **Error Message**: Displayed Below the Grid
                                if let Some(tweak_entry) = self.tweaks.get(&tweak_id) {
                                    if let TweakStatus::Failed(ref err) = tweak_entry.status {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("Error: {}", err),
                                        );
                                    }
                                }
                                // add vertical spacing
                                ui.add_space(UI_SPACING);
                            });
                        });
                    });

                    // **Second Column**: Widget (Toggle or Button), Fixed Width and Top-Aligned
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        // Ensure the widget has a fixed width to prevent shifting
                        let widget_width = BUTTON_DIMENSIONS[0]; // Adjust as per your BUTTON_DIMENSIONS
                        ui.set_width(widget_width);

                        match tweak_widget {
                            TweakWidget::Toggle => self.draw_toggle_widget(ui, tweak_id),
                            TweakWidget::Button => self.draw_button_widget(ui, tweak_id),
                        }
                    });

                    // End the row for the current tweak
                    ui.end_row();
                });
        }
    }

    fn draw_toggle_widget(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        if let Some(tweak_entry) = self.tweaks.get_mut(&tweak_id) {
            let mut is_enabled = tweak_entry.enabled;
            let has_error = matches!(tweak_entry.status, TweakStatus::Failed(_));

            let widget = ToggleSwitch::new(&mut is_enabled).with_error(has_error);

            let response_toggle = ui.add(widget);

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
                        // Add a dialog to inform the user about the task submission failure
                        self.dialogs.add(DialogDetails::new(
                            StandardDialog::error(
                                "Task Submission Error",
                                format!("Failed to submit task for tweak {:?}: {}", tweak_id, e),
                            )
                            .buttons(vec![("OK".into(), StandardReply::Ok)]),
                        ));
                    }
                }
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
            let response_button = ui.add(ActionButton::new(&mut button_state_mut));

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
                        // Add a dialog to inform the user about the task submission failure
                        self.dialogs.add(DialogDetails::new(
                            StandardDialog::error(
                                "Task Submission Error",
                                format!("Failed to submit task for tweak {:?}: {}", tweak_id, e),
                            )
                            .buttons(vec![("OK".into(), StandardReply::Ok)]),
                        ));
                    }
                }
            }

            if let TweakStatus::Failed(ref err) = tweak_entry.status {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
        }
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(TWEAK_CONTAINER_HEIGHT)
            .max_height(TWEAK_CONTAINER_HEIGHT)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin::same(UI_PADDING))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("v0.1.6a")
                                    .font(FontId::proportional(LABEL_FONT_SIZE)),
                            );
                            ui.separator();

                            let pending_reboot_count = self.count_tweaks_pending_reboot();

                            ui.label(
                                RichText::new(format!(
                                    "{} tweak{} pending restart",
                                    pending_reboot_count,
                                    if pending_reboot_count != 1 { "s" } else { "" }
                                ))
                                .font(FontId::proportional(LABEL_FONT_SIZE)),
                            );

                            ui.separator();
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            Button::new("Reboot into BIOS")
                                                .min_size(BUTTON_DIMENSIONS),
                                        )
                                        .clicked()
                                    {
                                        match reboot_into_bios() {
                                            Ok(_) => {
                                                tracing::debug!("Rebooting into BIOS settings...");
                                                self.cleanup();
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to reboot into BIOS: {:?}",
                                                    e
                                                );
                                                // Add a dialog to inform the user about the failure
                                                self.dialogs.add(
                                                    DialogDetails::new(
                                                        StandardDialog::error(
                                                            "Reboot Error",
                                                            format!(
                                                                "Failed to reboot into BIOS: {:?}",
                                                                e
                                                            ),
                                                        )
                                                        .buttons(vec![(
                                                            "OK".into(),
                                                            StandardReply::Ok,
                                                        )]),
                                                    ),
                                                );
                                            }
                                        }
                                    }

                                    if ui
                                        .add(
                                            Button::new("Restart Windows")
                                                .min_size(BUTTON_DIMENSIONS),
                                        )
                                        .clicked()
                                    {
                                        self.cleanup();
                                        if let Err(e) = reboot_system() {
                                            tracing::error!("Failed to initiate reboot: {:?}", e);
                                            // Add a dialog to inform the user about the failure
                                            self.dialogs.add(DialogDetails::new(
                                                StandardDialog::error(
                                                    "Reboot Error",
                                                    format!("Failed to initiate reboot: {:?}", e),
                                                )
                                                .buttons(vec![("OK".into(), StandardReply::Ok)]),
                                            ));
                                        }
                                    }
                                },
                            );
                        });
                    });
            });
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Process dialogs first
        if !self.dialogs.dialogs().is_empty() {
            if let Some(res) = self.dialogs.show(ctx) {
                // handle reply from close confirmation dialog
                if let Ok(StandardReply::Ok) = res.reply() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        } else {
            self.update_tweak_states();

            if !self.initial_states_loaded {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_space(200.0);
                            ui.heading("Reading system state...");
                            ui.add(egui::widgets::Spinner::new());
                        });
                });
            } else {
                self.draw_status_bar(ctx);

                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.draw_ui(ui);
                        });
                });
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cleanup();
    }
}

fn main() -> eframe::Result<()> {
    // Initialize logging based on build mode
    #[cfg(debug_assertions)]
    {
        // Initialize tracing to log to terminal (stdout) in debug mode
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_target(false)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        // In release mode, set up a no-op subscriber to disable logging
        // This prevents any tracing macros from producing output
        use tracing_subscriber::Registry;
        let noop_subscriber = Registry::default();
        tracing::subscriber::set_global_default(noop_subscriber)
            .expect("Failed to set global subscriber.");
    }

    // Set up eframe NativeOptions as before
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT / 2.0]),
        ..Default::default()
    };

    let run_span = tracing::span!(Level::INFO, "Run Native");
    run_span.in_scope(|| {
        eframe::run_native(
            "OC Tool",
            options,
            Box::new(|cc| Ok(Box::new(MyApp::new(cc)))), // No guard needed
        )
    })
}
