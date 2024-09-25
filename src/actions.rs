// src/actions.rs

use crate::{models::Tweak, tweaks::TweakMethod};

// Trait defining the apply and revert methods
pub trait TweakAction {
    fn read(&self) -> Result<(), anyhow::Error>;
    fn apply(&self) -> Result<(), anyhow::Error>;
    fn revert(&self) -> Result<(), anyhow::Error>;
}

// Implement TweakAction for Tweak
impl TweakAction for Tweak {
    fn read(&self) -> Result<(), anyhow::Error> {
        match &self.config {
            TweakMethod::Registry(config) => {
                config.read_current_value()?;
            }
            TweakMethod::GroupPolicy(config) => {
                config.read_current_value()?;
            }
            TweakMethod::Command(_) => {
                // For CommandTweaks, read can be a no-op
            }
        }
        Ok(())
    }

    fn apply(&self) -> Result<(), anyhow::Error> {
        match &self.config {
            TweakMethod::Registry(config) => {
                config.apply_registry_tweak()?;
            }
            TweakMethod::GroupPolicy(config) => {
                config.apply_group_policy_tweak()?;
            }
            TweakMethod::Command(config) => {
                config.apply()?;
            }
        }
        Ok(())
    }

    fn revert(&self) -> Result<(), anyhow::Error> {
        match &self.config {
            TweakMethod::Registry(config) => {
                config.revert_registry_tweak()?;
            }
            TweakMethod::GroupPolicy(config) => {
                config.revert_group_policy_tweak()?;
            }
            TweakMethod::Command(_) => {
                // Typically, commands cannot be reverted, so you can leave this empty or return an error
            }
        }
        Ok(())
    }
}
