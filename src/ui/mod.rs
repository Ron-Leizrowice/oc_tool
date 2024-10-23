// src/ui/mod.rs

pub mod button;
pub mod combobox;
pub mod container;
pub mod switch;

/// Enum representing the different widget types for a tweak.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TweakWidget {
    Toggle,
    Button,
}
