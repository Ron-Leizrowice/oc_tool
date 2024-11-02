// src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::collections::BTreeMap;

use anyhow::Context;
use eframe::{egui, App, Frame, NativeOptions};
use egui::{Button, FontFamily, FontId, RichText};
use egui_dialogs::{DialogDetails, Dialogs, StandardDialog, StandardReply};
use oc_tool::{
    constants::{
        LABEL_FONT_SIZE, TWEAK_CONTAINER_HEIGHT, TWEAK_CONTAINER_WIDTH, UI_PADDING, UI_SPACING,
        WINDOW_HEIGHT, WINDOW_WIDTH,
    },
    orchestrator::{TaskOrchestrator, TweakAction, TweakTask},
    tweaks::{self, Tweak, TweakCategory, TweakId, TweakOption, TweakStatus},
    ui::{
        button::{ActionButton, ButtonState, BUTTON_DIMENSIONS},
        combobox::SettingsComboBox,
        switch::ToggleSwitch,
        TweakWidget,
    },
    utils::{
        windows::{is_elevated, reboot_into_bios, reboot_system},
        winring0::{
            setup_winring0_driver, verify_winring0_driver, WINRING0_DRIVER,
            WINRING0_SETUP_USER_MESSAGE,
        },
    },
};
use tracing::Level;
use tracing_subscriber::{self};

pub struct MyApp {
    /// A map of all tweaks available in the application, indexed by their unique ID
    /// The tweaks are stored in a BTreeMap to ensure consistent ordering
    /// Each Tweak object contains the method for applying/reverting/reading initial state of the tweak
    /// and the current state of the tweak
    pub tweaks: BTreeMap<TweakId, Tweak<'static>>,

    /// Task orchestrator to manage tweak tasks
    pub orchestrator: TaskOrchestrator,

    // State tracking for initial state reads
    pub initial_states_loaded: bool,
    pub pending_initial_state_reads: usize,

    pub dialogs: Dialogs<'static>,
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
                    .buttons(vec![("OK".into(), StandardReply::Cancel)]),
            ));
        } else {
            // Attempt to verify WinRing0 setup
            match verify_winring0_driver() {
                Ok(_) => {
                    tracing::debug!("WinRing0 driver already set up.");
                }
                Err(err) => {
                    tracing::debug!("WinRing0 driver not set up: {:?}", err);
                    dialogs.add(DialogDetails::new(
                        StandardDialog::confirm(
                            "WinRing0 Driver Setup",
                            WINRING0_SETUP_USER_MESSAGE,
                        )
                        .buttons(vec![
                            ("Accept".into(), StandardReply::Yes),
                            ("Cancel".into(), StandardReply::Cancel),
                        ]),
                    ));
                }
            }
        }

        // Submit initial state read tasks for all tweaks
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

        Self {
            tweaks,
            orchestrator,
            initial_states_loaded: false,
            pending_initial_state_reads,
            dialogs,
        }
    }

    /// Iterates through all the tweaks and checks how many are waiting on a reboot
    fn count_tweaks_pending_reboot(&self) -> usize {
        self.tweaks
            .iter()
            .filter(|(_, tweak)| tweak.pending_reboot)
            .count()
    }

    /// Updates the tweak states based on the results received from the orchestrator
    fn update_tweak_states(&mut self) -> anyhow::Result<()> {
        while let Some(result) = self.orchestrator.try_recv_result() {
            if let Some(tweak) = self.tweaks.get_mut(&result.id) {
                if result.success {
                    match result.action {
                        TweakAction::Set(ref option) => {
                            // Update the state with the new option if successful
                            tweak.state = option.clone();
                            tweak.status = TweakStatus::Idle;
                            tracing::debug!(
                                "Successfully updated tweak {:?} to state {:?}",
                                result.id,
                                option
                            );
                        }
                        TweakAction::Enable => {
                            tweak.state = TweakOption::Enabled(true);
                            tweak.status = TweakStatus::Idle;
                            tracing::debug!("Successfully enabled tweak {:?}", result.id);
                        }
                        TweakAction::Disable => {
                            tweak.state = TweakOption::Enabled(false);
                            tweak.status = TweakStatus::Idle;
                            tracing::debug!("Successfully disabled tweak {:?}", result.id);
                        }
                        TweakAction::ReadInitialState => {
                            // Update the tweak state with the initial state read
                            tweak.state = result.state.clone().unwrap();
                            tweak.status = TweakStatus::Idle;
                            // Decrement the pending reads count
                            if self.pending_initial_state_reads > 0 {
                                self.pending_initial_state_reads -= 1;
                            }
                            tracing::debug!(
                                "Successfully read initial state for tweak {:?}: {:?}",
                                result.id,
                                result.state.unwrap()
                            );
                            // If all reads are done, mark initialization as complete
                            if self.pending_initial_state_reads == 0 {
                                self.initial_states_loaded = true;
                            }
                        }
                    }
                } else {
                    // Handle failure
                    if let Some(err) = result.error {
                        tweak.status = TweakStatus::Failed(err.to_string());
                        tracing::error!("Failed to update tweak {:?}: {:?}", result.id, err);

                        // Attempt to read the current state again
                        if let Err(e) = self.orchestrator.submit_task(TweakTask {
                            id: result.id,
                            method: tweak.method.clone(),
                            action: TweakAction::ReadInitialState,
                        }) {
                            tracing::error!("Failed to submit state read task: {:?}", e);
                        }
                    }
                }
            }
        }

        // Optionally, check if all initial reads are done outside the loop
        if self.pending_initial_state_reads == 0 && !self.initial_states_loaded {
            self.initial_states_loaded = true;
            tracing::debug!("All initial state reads completed.");
        }

        Ok(())
    }

    fn cleanup(&mut self) {
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
                    // First Column: Collapsing Header with Description
                    ui.vertical(|ui| {
                        ui.collapsing(tweak_name, |ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(tweak_description)
                                        .font(FontId::new(12.0, FontFamily::Proportional)),
                                );
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

                    // Second Column: Widget (Toggle, Button, or ComboBox)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        // Set fixed width for all widgets
                        let widget_width = BUTTON_DIMENSIONS[0];
                        ui.set_width(widget_width);

                        match tweak_widget {
                            TweakWidget::Toggle => self.draw_toggle_widget(ui, tweak_id),
                            TweakWidget::Button => self.draw_button_widget(ui, tweak_id),
                            TweakWidget::SettingsComboBox => {
                                // Ensure combo box aligns with other widgets
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::TOP),
                                    |ui| self.draw_combo_box_widget(ui, tweak_id),
                                );
                            }
                        }
                    });

                    // End the row for the current tweak
                    ui.end_row();
                });
        }
    }

    fn draw_toggle_widget(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        if let Some(tweak) = self.tweaks.get_mut(&tweak_id) {
            let mut is_enabled = tweak.state == TweakOption::Enabled(true);
            let has_error = matches!(tweak.status, TweakStatus::Failed(_));

            let widget = ToggleSwitch::new(&mut is_enabled).with_error(has_error);

            let response_toggle = ui.add(widget);

            if response_toggle.changed() {
                tweak.state = if is_enabled {
                    TweakOption::Enabled(true)
                } else {
                    TweakOption::Enabled(false)
                };
                if tweak.requires_reboot {
                    tweak.pending_reboot = true;
                }
                tweak.status = TweakStatus::Busy;
                let result = self.orchestrator.submit_task(TweakTask {
                    id: tweak_id,
                    method: tweak.method.clone(),
                    action: if is_enabled {
                        TweakAction::Enable
                    } else {
                        TweakAction::Disable
                    },
                });
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        tweak.status = TweakStatus::Failed(e.to_string());
                    }
                }
            }
        }
    }

    fn draw_button_widget(&mut self, ui: &mut egui::Ui, tweak_id: TweakId) {
        if let Some(tweak_entry) = self.tweaks.get_mut(&tweak_id) {
            let button_state = match tweak_entry.status {
                TweakStatus::Idle | TweakStatus::Failed(_) => ButtonState::Default,
                TweakStatus::Busy => ButtonState::InProgress,
            };

            let mut button_state_mut = button_state;
            let response_button = ui.add(ActionButton::new(&mut button_state_mut));

            if response_button.clicked() && button_state == ButtonState::Default {
                tweak_entry.status = TweakStatus::Busy;
                let result = self.orchestrator.submit_task(TweakTask {
                    id: tweak_id,
                    method: tweak_entry.method.clone(),
                    action: TweakAction::Enable,
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

    fn draw_combo_box_widget(
        &mut self,
        ui: &mut egui::Ui,
        tweak_id: TweakId,
    ) -> anyhow::Result<()> {
        if let Some(tweak) = self.tweaks.get_mut(&tweak_id) {
            // Extract the list of options (keys) from the BTreeMap
            let options: Vec<TweakOption> = tweak.options.to_vec();

            // Determine the currently selected option's index
            let mut selected_option = options
                .iter()
                .position(|option| option == &tweak.state)
                .context("Failed to find selected option index.")?;

            // Create the combo box widget with the list of options
            let widget = SettingsComboBox::new(tweak_id, &mut selected_option, options.clone());
            let response_combo = ui.add(widget);

            if response_combo.changed() {
                tracing::debug!(
                    "Selected option for tweak {:?}: {:?}",
                    tweak_id,
                    selected_option
                );
                // Retrieve the newly selected option based on the index
                let new_option = options[selected_option].clone();

                tweak.state = new_option.clone();
                if tweak.requires_reboot {
                    tweak.pending_reboot = true;
                }

                // Submit the task with the new option
                self.orchestrator
                    .submit_task(TweakTask {
                        id: tweak_id,
                        method: tweak.method.clone(),
                        action: TweakAction::Set(new_option),
                    })
                    .context("Failed to submit registry task for tweak.")?;

                tweak.status = TweakStatus::Busy;
            }
        }
        Ok(())
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
                                RichText::new("v0.1.7a")
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
                match res.reply() {
                    Ok(StandardReply::Cancel) => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    Ok(StandardReply::Yes) => match setup_winring0_driver() {
                        Ok(_) => {
                            tracing::debug!("WinRing0 driver setup successful.");
                        }
                        Err(e) => {
                            tracing::error!("Failed to set up WinRing0 driver: {:?}", e);
                            self.dialogs.add(DialogDetails::new(
                                StandardDialog::error(
                                    "Driver Setup Error",
                                    format!("Failed to set up WinRing0 driver: {:?}", e),
                                )
                                .buttons(vec![("OK".into(), StandardReply::Ok)]),
                            ));
                        }
                    },
                    _ => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        } else {
            match self.update_tweak_states() {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Failed to update tweak states: {:?}", e);
                }
            }

            if !self.initial_states_loaded {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Reading system state...");
                    ui.add(egui::widgets::Spinner::new());
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
