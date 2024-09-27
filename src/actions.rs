// src/actions.rs

use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    tweaks::{TweakId, TweakMethod},
    widgets::TweakWidget,
};

/// Represents a single tweak that can be applied to the system.
#[derive(Debug, Clone)]
pub struct Tweak {
    /// Unique identifier for the tweak.
    pub id: TweakId,
    /// Display name of the tweak.
    pub name: String,
    /// Description of what the tweak does.
    pub description: String,
    /// The type of UI widget associated with the tweak (e.g., Switch, Button).
    pub widget: TweakWidget,
    /// Indicates whether applying this tweak requires a system restart.
    pub requires_restart: bool,
    /// The method used to apply/revert the tweak (Registry, Group Policy, Command).
    pub method: TweakMethod,
    /// Indicates whether the tweak is currently enabled.
    pub enabled: Arc<AtomicBool>,
    /// The status of the tweak (e.g., "Applied", "In Progress", "Failed").
    pub status: TweakStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweakStatus {
    Idle,
    Applying,
    Failed(String),
}

impl Tweak {
    pub fn new(
        id: TweakId,
        name: String,
        description: String,
        method: TweakMethod,
        widget: TweakWidget,
        requires_restart: bool,
    ) -> Self {
        Self {
            id,
            name,
            description,
            widget,
            requires_restart,
            method,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
        }
    }
}

/// Trait defining actions that can be performed on a tweak, such as checking if it's enabled,
/// applying the tweak, and reverting it.
pub trait TweakAction {
    /// Determines if the tweak is currently enabled.
    fn is_enabled(&self) -> Result<bool, anyhow::Error>;

    /// Applies the tweak.
    fn apply(&self) -> Result<(), anyhow::Error>;

    /// Reverts the tweak to its default state.
    fn revert(&self) -> Result<(), anyhow::Error>;
}

impl TweakAction for Tweak {
    /// Checks the current state of the tweak and updates the `enabled` field accordingly.
    fn is_enabled(&self) -> Result<bool, anyhow::Error> {
        match &self.method {
            TweakMethod::Registry(method) => Ok(method.is_registry_tweak_enabled()?),
            TweakMethod::GroupPolicy(config) => Ok(config.is_group_policy_tweak_enabled()?),
            TweakMethod::Powershell(config) => Ok(config.is_powershell_script_enabled()?),
        }
    }

    /// Applies the tweak based on its method.
    fn apply(&self) -> Result<(), anyhow::Error> {
        match &self.method {
            TweakMethod::Registry(config) => {
                config.apply_registry_tweak()?;
            }
            TweakMethod::GroupPolicy(config) => {
                config.apply_group_policy_tweak()?;
            }
            TweakMethod::Powershell(config) => {
                config.run_apply_script()?;
            }
        }
        Ok(())
    }

    /// Reverts the tweak to its default state based on its method.
    fn revert(&self) -> Result<(), anyhow::Error> {
        match &self.method {
            TweakMethod::Registry(method) => {
                method.revert_registry_tweak()?;
            }
            TweakMethod::GroupPolicy(method) => {
                method.revert_group_policy_tweak()?;
            }
            TweakMethod::Powershell(method) => {
                if method.undo_script.is_some() {
                    method.run_undo_script()?
                }
            }
        }
        Ok(())
    }
}
