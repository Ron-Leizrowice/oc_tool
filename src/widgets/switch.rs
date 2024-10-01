// src/widgets/switch.rs

use egui::{self, Sense};

/// A simple toggle switch widget.
///
/// # Parameters
/// - `on`: A mutable reference to a boolean value that indicates the state of the switch.
///
/// # Returns
/// An [`egui::Widget`] that can be used to render the toggle switch.
pub fn toggle_switch(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| {
        // Create the ToggleSwitch widget.
        let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

        // Handle click events.
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }

        // Provide widget information for accessibility.
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), false, "")
        });

        // Draw the toggle based on the current state.
        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
            let visuals = ui.style().interact_selectable(&response, *on);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = egui::pos2(circle_x, rect.center().y);
            ui.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
        }

        response
    }
}
