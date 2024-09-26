// src/widgets/switch.rs

use egui::{self, Color32, Pos2, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToggleSwitchState {
    Off,
    InProgress,
    On,
}

#[derive(Clone, Copy, Debug)]
pub struct ToggleSwitch {
    pub state: ToggleSwitchState,
    stroke_color: Stroke,
    rounding: Rounding,
    min_size: Vec2,
}

impl Default for ToggleSwitch {
    fn default() -> Self {
        Self::new(ToggleSwitchState::Off)
    }
}

impl ToggleSwitch {
    pub fn new(state: ToggleSwitchState) -> Self {
        Self {
            state,
            stroke_color: Stroke::new(1.0, Color32::BLACK),
            rounding: Rounding::same(15.0),
            min_size: Vec2::new(60.0, 30.0),
        }
    }

    pub fn fill_color(&self) -> Color32 {
        match self.state {
            ToggleSwitchState::Off => Color32::from_rgb(200, 200, 200),
            ToggleSwitchState::InProgress => Color32::from_rgb(100, 150, 250),
            ToggleSwitchState::On => Color32::from_rgb(100, 200, 100),
        }
    }
}

impl Widget for ToggleSwitch {
    fn ui(self, ui: &mut Ui) -> Response {
        let is_clickable = matches!(self.state, ToggleSwitchState::Off | ToggleSwitchState::On);

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
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), false, "")
        });

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let expanded_rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();

            // Draw the background rectangle
            ui.painter().rect(
                expanded_rect,
                self.rounding,
                self.fill_color(),
                self.stroke_color,
            );

            // Position of the toggle circle
            let circle_pos = match self.state {
                ToggleSwitchState::Off => Pos2::new(rect.left() + radius, rect.center().y),
                ToggleSwitchState::InProgress => rect.center(),
                ToggleSwitchState::On => Pos2::new(rect.right() - radius, rect.center().y),
            };

            // Circle fill color
            let circle_fill = Color32::WHITE;

            // Draw the toggle circle
            ui.painter()
                .circle(circle_pos, 0.75 * radius, circle_fill, visuals.fg_stroke);
        }

        response
    }
}
