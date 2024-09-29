// src/widgets/switch.rs

use egui::{self, Color32, Pos2, Response, Sense, Ui, Widget};

/// Custom ToggleSwitch widget.
#[derive(Clone, Copy, Debug)]
pub struct ToggleSwitch;

impl Widget for ToggleSwitch {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

        // Handle click events.
        if response.clicked() {
            response.mark_changed();
        }

        // Provide widget information for accessibility.
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), false, "")
        });

        // Only paint if the widget is visible.
        if ui.is_rect_visible(rect) {
            // Note: The `on` state is managed externally and should be passed to the widget.
            // Therefore, the widget should not attempt to retrieve it from UI memory.

            // Instead, the caller manages the state and updates it based on user interaction.
        }

        response
    }
}

/// Helper function to create a ToggleSwitch widget bound to a boolean state.
///
/// ## Example:
/// ```ignore
/// let mut my_bool = true;
/// ui.add(toggle_switch(&mut my_bool));
/// ```
pub fn toggle_switch(on: &mut bool) -> impl Widget + '_ {
    move |ui: &mut Ui| {
        // Create the ToggleSwitch widget.
        let toggle = ToggleSwitch;

        // Allocate the widget and get the response.
        let response = ui.add(toggle);

        // If the widget was clicked, toggle the state.
        if response.clicked() {
            *on = !*on;
        }

        // Draw the toggle based on the current state.
        if ui.is_rect_visible(response.rect) {
            let rect = response.rect;
            let radius = rect.height() / 2.0;

            // Define colors based on the toggle state.
            let track_color = if *on {
                Color32::from_rgb(0, 200, 0) // Green for "On"
            } else {
                Color32::from_rgb(200, 200, 200) // Grey for "Off"
            };

            let knob_color = Color32::WHITE;

            // Draw the track.
            ui.painter().rect_filled(rect, radius, track_color);

            // Calculate knob position.
            let knob_x = if *on {
                rect.right() - radius
            } else {
                rect.left() + radius
            };
            let knob_center = Pos2::new(knob_x, rect.center().y);

            // Draw the knob.
            ui.painter()
                .circle_filled(knob_center, radius * 0.75, knob_color);
        }

        response
    }
}
