// src/widgets/button.rs

use eframe::egui::{FontId, Response, Rounding, Sense, Ui};
use egui::{vec2, Vec2};

pub const BUTTON_DIMENSIONS: Vec2 = vec2(35.0, 20.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Default,
    InProgress,
}

pub struct ActionButton<'a> {
    state: &'a mut ButtonState,
}

impl<'a> ActionButton<'a> {
    pub fn new(state: &'a mut ButtonState) -> Self {
        Self { state }
    }
}

impl eframe::egui::Widget for ActionButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        // Determine the label based on the current state
        let label = match *self.state {
            ButtonState::Default => "Run",
            ButtonState::InProgress => "...",
        };

        // Determine if the button is clickable
        let is_clickable = matches!(*self.state, ButtonState::Default);

        // Set the interaction sense based on the button state
        let sense = if is_clickable {
            Sense::click()
        } else {
            Sense::hover()
        };

        // Allocate space for the button
        let (rect, mut response) = ui.allocate_exact_size(BUTTON_DIMENSIONS, sense);

        // Handle button clicks
        if is_clickable && response.clicked() {
            *self.state = ButtonState::InProgress;
            response.mark_changed();
        }

        // Render the button if it's visible
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            // Draw the button background
            ui.painter()
                .rect_filled(rect, Rounding::same(5.0), visuals.bg_fill);
            ui.painter()
                .rect_stroke(rect, Rounding::same(5.0), visuals.bg_stroke);

            // Layout the text centered within the button
            let galley = ui.fonts(|fonts| {
                fonts.layout_no_wrap(
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
