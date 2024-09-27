// src/widgets/button.rs

use egui::{self, Color32, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget};

const DEFAULT_TEXT: &str = "Apply";
const IN_PROGRESS_TEXT: &str = "Applying...";
const COMPLETED_TEXT: &str = "Done";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Default,
    InProgress,
}

#[derive(Clone, Debug)]
pub struct ApplyButton {
    pub state: ButtonState,
    fill: Color32,
    stroke: Stroke,
    rounding: Rounding,
    min_size: Vec2,
}

impl Default for ApplyButton {
    fn default() -> Self {
        Self::new(ButtonState::Default)
    }
}

impl ApplyButton {
    pub fn new(state: ButtonState) -> Self {
        Self {
            state,
            fill: Color32::from_rgb(100, 150, 250),
            stroke: Stroke::new(1.0, Color32::BLACK),
            rounding: Rounding::same(5.0),
            min_size: Vec2::new(100.0, 30.0),
        }
    }
}

impl Widget for ApplyButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let label = match self.state {
            ButtonState::Default => DEFAULT_TEXT,
            ButtonState::InProgress => IN_PROGRESS_TEXT,
        };

        let is_clickable = matches!(self.state, ButtonState::Default);

        let sense = if is_clickable {
            Sense::click()
        } else {
            Sense::hover()
        };

        let (rect, mut response) = ui.allocate_exact_size(self.min_size, sense);

        if is_clickable && response.clicked() {
            response.mark_changed();
        }

        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), false, label)
        });

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            // Draw the button background
            ui.painter().rect_filled(rect, self.rounding, self.fill);
            ui.painter().rect_stroke(rect, self.rounding, self.stroke);

            // Center the text
            let galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    label.to_string(),
                    egui::FontId::default(),
                    visuals.text_color(),
                )
            });

            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, visuals.text_color());
        }

        response
    }
}
