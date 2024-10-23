use eframe::egui::{lerp, pos2, vec2, Color32, Response, Sense, Ui};
use egui::Vec2;

const TOGGLE_DIMENSIONS: Vec2 = vec2(2.0, 1.0);

pub struct ToggleSwitch<'a> {
    state: &'a mut bool,
    has_error: bool,
}

impl<'a> ToggleSwitch<'a> {
    pub fn new(state: &'a mut bool) -> Self {
        Self {
            state,
            has_error: false,
        }
    }

    pub fn with_error(mut self, has_error: bool) -> Self {
        self.has_error = has_error;
        self
    }
}

impl eframe::egui::Widget for ToggleSwitch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        // Create the ToggleSwitch widget.
        let desired_size = ui.spacing().interact_size.y * TOGGLE_DIMENSIONS;
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

        // Handle click events.
        if response.clicked() {
            *self.state = !*self.state;
            response.mark_changed();
        }

        // Draw the toggle based on the current state.
        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool_responsive(response.id, *self.state);
            let mut visuals = ui.style().interact_selectable(&response, *self.state);

            // Modify the visuals if there's an error
            if self.has_error {
                visuals.bg_fill = Color32::from_rgb(200, 0, 0); // Dark red background
                visuals.fg_stroke.color = Color32::WHITE; // White circle
            }

            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();

            // Draw the background
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);

            // Draw the circle
            let circle_x = lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = pos2(circle_x, rect.center().y);
            ui.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
        }

        response
    }
}
