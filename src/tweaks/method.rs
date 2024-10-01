// src/tweaks/method.rs

use anyhow::Error;

use super::TweakId;

/// Trait defining the behavior for all tweak methods.
pub trait TweakMethod: Send + Sync {
    /// Checks if the tweak is currently enabled.
    fn initial_state(&self, id: TweakId) -> Result<bool, Error>;

    /// Applies the tweak.
    fn apply(&self, id: TweakId) -> Result<(), Error>;

    /// Reverts the tweak.
    fn revert(&self, id: TweakId) -> Result<(), Error>;
}
