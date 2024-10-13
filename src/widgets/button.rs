// src/widgets/button.rs

use egui::{self, FontId, Rounding, Sense};

const DEFAULT_TEXT: &str = "Run";
const IN_PROGRESS_TEXT: &str = "...";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Default,
    InProgress,
}

pub fn action_button(state: &mut ButtonState) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| {
        // Determine the label based on the current state
        let label = match *state {
            ButtonState::Default => DEFAULT_TEXT,
            ButtonState::InProgress => IN_PROGRESS_TEXT,
        };

        // Determine if the button is clickable
        let is_clickable = matches!(*state, ButtonState::Default);

        // Set the interaction sense based on the button state
        let sense = if is_clickable {
            Sense::click()
        } else {
            Sense::hover()
        };

        // Allocate space for the button
        let (rect, mut response) = ui.allocate_exact_size([35.0, 20.0].into(), sense);

        // Handle button clicks
        if is_clickable && response.clicked() {
            *state = ButtonState::InProgress;
            response.mark_changed();
        }

        // Provide widget information for accessibility and debugging
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), false, label)
        });

        // Render the button if it's visible
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            // Draw the button background
            ui.painter()
                .rect_filled(rect, Rounding::same(5.0), visuals.bg_fill);
            ui.painter()
                .rect_stroke(rect, Rounding::same(5.0), visuals.bg_stroke);

            // Layout the text centered within the button
            let galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    label.to_string(),
                    FontId::proportional(12.0),
                    visuals.text_color(),
                )
            });

            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley, visuals.text_color());
        }

        response
    }
}
