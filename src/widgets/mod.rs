// src/widgets/mod.rs

use button::ApplyButton;
use switch::ToggleSwitch;

pub mod button;
pub mod switch;

/// Enum representing the different widget types for a tweak.
#[derive(Clone, Debug)]
pub enum TweakWidget {
    Switch(ToggleSwitch),
    Button(ApplyButton),
}
