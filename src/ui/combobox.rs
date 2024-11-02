use eframe::egui::{self, ComboBox as EguiComboBox, Response, Widget};
use egui::FontId;

use crate::{
    constants::LABEL_FONT_SIZE,
    tweaks::{TweakId, TweakOption},
};

/// A simple wrapper around egui's ComboBox
pub struct SettingsComboBox<'a> {
    id_source: String,
    selected_index: &'a mut usize,
    options_text: Vec<String>,
}

impl<'a> SettingsComboBox<'a> {
    /// Create a new ComboBox with a unique ID
    pub fn new(
        tweak_id: TweakId,
        selected_index: &'a mut usize,
        options: Vec<TweakOption>,
    ) -> Self {
        Self {
            id_source: format!("combo_box_{:?}", tweak_id),
            selected_index,
            options_text: options
                .into_iter()
                .map(|option| match option {
                    TweakOption::Option(text) => text,
                    _ => "Unknown".to_string(),
                })
                .collect(),
        }
    }
}

impl Widget for SettingsComboBox<'_> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let Self {
            id_source,
            selected_index,
            options_text: options,
        } = self;

        let selected_text = options.get(*selected_index).cloned().unwrap();

        let original_selected_index = *selected_index;

        let mut combo = EguiComboBox::from_id_salt(id_source).selected_text(selected_text.clone());

        // set the combo box width to the text width
        let text_width = ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(
                    selected_text,
                    FontId::proportional(LABEL_FONT_SIZE),
                    egui::Color32::WHITE,
                )
                .size()
                .x
        });
        combo = combo.width(text_width);

        let mut response = combo
            .show_ui(ui, |ui| {
                for (idx, option) in options.iter().enumerate() {
                    ui.selectable_value(selected_index, idx, option);
                }
            })
            .response;

        if *selected_index != original_selected_index {
            response.mark_changed();
        }

        response
    }
}
