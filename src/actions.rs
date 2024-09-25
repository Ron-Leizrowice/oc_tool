// src/actions.rs

use crate::{models::Tweak, tweaks::TweakMethod};

// Trait defining the apply and revert methods
pub trait TweakAction {
    fn is_enabled(&mut self) -> Result<(), anyhow::Error>;
    fn apply(&mut self) -> Result<(), anyhow::Error>;
    fn revert(&mut self) -> Result<(), anyhow::Error>;
}

// Implement TweakAction for Tweak
impl TweakAction for Tweak {
    fn is_enabled(&mut self) -> Result<(), anyhow::Error> {
        match &self.method {
            TweakMethod::Registry(config) => {
                self.enabled = config.read_current_value()? == config.default_value;
                Ok(())
            }
            TweakMethod::GroupPolicy(config) => {
                self.enabled = config.read_current_value()? == config.default_value;
                Ok(())
            }
            TweakMethod::Command(config) => match &config.default {
                None => {
                    self.enabled = false;
                    tracing::debug!(
                        "Command tweak '{}': No target state defined. Setting enabled to false.",
                        self.name
                    );
                    Ok(())
                }
                Some(target) => {
                    tracing::debug!(
                        "Command tweak '{}': Reading current state to compare with target state.",
                        self.name
                    );
                    let current_state = match config.read_current_state() {
                        Ok(state) => state.unwrap(),
                        Err(e) => {
                            tracing::debug!(
                                "Failed to read current state for command tweak '{}': {}",
                                self.name,
                                e
                            );
                            return Err(e);
                        }
                    };
                    tracing::debug!(
                        "Command tweak '{}': Current state: {:?}",
                        self.name,
                        current_state
                    );
                    tracing::debug!("Command tweak '{}': Target state: {:?}", self.name, target);
                    self.enabled = current_state == *target;
                    tracing::debug!(
                        "Command tweak '{}': Enabled set to {}",
                        self.name,
                        self.enabled
                    );
                    Ok(())
                }
            },
        }
    }

    fn apply(&mut self) -> Result<(), anyhow::Error> {
        match &self.method {
            TweakMethod::Registry(config) => {
                config.apply_registry_tweak()?;
            }
            TweakMethod::GroupPolicy(config) => {
                config.apply_group_policy_tweak()?;
            }
            TweakMethod::Command(config) => {
                config.run_apply_script()?;
            }
        }
        Ok(())
    }

    fn revert(&mut self) -> Result<(), anyhow::Error> {
        match &self.method {
            TweakMethod::Registry(method) => {
                method.revert_registry_tweak()?;
            }
            TweakMethod::GroupPolicy(method) => {
                method.revert_group_policy_tweak()?;
            }
            TweakMethod::Command(method) => match method.undo_script {
                Some(_) => method.run_undo_script()?,
                None => {
                    tracing::debug!(
                        "Command tweak '{}': No undo script defined. Skipping revert.",
                        self.name
                    );
                }
            },
        }
        Ok(())
    }
}
