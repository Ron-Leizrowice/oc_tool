// src/widgets/mod.rs

pub mod button;
pub mod switch;

/// Enum representing the different widget types for a tweak.
#[derive(Clone, Debug)]
pub enum TweakWidget {
    Switch,
    Button,
}
