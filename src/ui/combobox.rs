use eframe::egui::{self, ComboBox as EguiComboBox, Response, Widget};

/// A simple wrapper around egui's ComboBox
pub struct SettingsComboBox<'a> {
    id_source: &'a str,
    selected_index: &'a mut usize,
    options: Vec<String>,
}

impl<'a> SettingsComboBox<'a> {
    /// Create a new ComboBox with a unique ID
    pub fn new(id_source: &'a str, selected_index: &'a mut usize, options: Vec<String>) -> Self {
        Self {
            id_source,
            selected_index,
            options,
        }
    }
}

impl Widget for SettingsComboBox<'_> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let Self {
            id_source,
            selected_index,
            options,
        } = self;

        let selected_text = options
            .get(*selected_index)
            .cloned()
            .unwrap_or_else(|| "Select...".to_string());

        let mut combo = EguiComboBox::from_id_salt(id_source).selected_text(selected_text);

        combo = combo.width(40.0);

        combo
            .show_ui(ui, |ui| {
                for (idx, option) in options.iter().enumerate() {
                    ui.selectable_value(selected_index, idx, option);
                }
            })
            .response
    }
}
