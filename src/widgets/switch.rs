// src/widgets/switch.rs

use eframe::egui::{lerp, pos2, vec2, Response, Sense, Ui};

pub struct ToggleSwitch<'a> {
    state: &'a mut bool,
}

impl<'a> ToggleSwitch<'a> {
    pub fn new(state: &'a mut bool) -> Self {
        Self { state }
    }
}

impl<'a> eframe::egui::Widget for ToggleSwitch<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        // Create the ToggleSwitch widget.
        let desired_size = ui.spacing().interact_size.y * vec2(2.0, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

        // Handle click events.
        if response.clicked() {
            *self.state = !*self.state;
            response.mark_changed();
        }

        // Draw the toggle based on the current state.
        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool_responsive(response.id, *self.state);
            let visuals = ui.style().interact_selectable(&response, *self.state);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = pos2(circle_x, rect.center().y);
            ui.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
        }

        response
    }
}
