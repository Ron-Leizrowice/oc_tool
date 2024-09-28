// src/actions.rs

use std::sync::{atomic::AtomicBool, Arc, Mutex};

use anyhow::Error;

use crate::{
    tweaks::{fetch_tweak_method, TweakId, TweakMethod},
    widgets::TweakWidget,
};

/// Represents the current status of a tweak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweakStatus {
    Idle,
    Applying,
    Failed(String),
}

/// Represents a single tweak that can be applied to the system.
#[derive(Debug, Clone)]
pub struct Tweak {
    /// Unique identifier for the tweak.
    pub id: TweakId,
    /// Indicates whether the tweak is currently enabled.
    pub enabled: Arc<AtomicBool>,
    /// The status of the tweak (e.g., "Applied", "In Progress", "Failed").
    pub status: TweakStatus,
    /// The widget to use for each tweak
    pub widget: TweakWidget,
}

pub fn initialize_all_tweaks() -> Vec<Arc<Mutex<Tweak>>> {
    vec![
        Arc::new(Mutex::new(Tweak {
            id: TweakId::LargeSystemCache,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
            widget: TweakWidget::Switch,
        })),
        Arc::new(Mutex::new(Tweak {
            id: TweakId::SystemResponsiveness,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
            widget: TweakWidget::Switch,
        })),
        Arc::new(Mutex::new(Tweak {
            id: TweakId::DisableHWAcceleration,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
            widget: TweakWidget::Switch,
        })),
        Arc::new(Mutex::new(Tweak {
            id: TweakId::Win32PrioritySeparation,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
            widget: TweakWidget::Switch,
        })),
        Arc::new(Mutex::new(Tweak {
            id: TweakId::DisableCoreParking,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
            widget: TweakWidget::Switch,
        })),
        Arc::new(Mutex::new(Tweak {
            id: TweakId::SeLockMemoryPrivilege,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
            widget: TweakWidget::Switch,
        })),
        Arc::new(Mutex::new(Tweak {
            id: TweakId::UltimatePerformancePlan,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
            widget: TweakWidget::Switch,
        })),
    ]
}

/// Trait defining actions that can be performed on a tweak, such as checking if it's enabled,
/// applying the tweak, and reverting it.
pub trait TweakAction {
    /// Determines if the tweak is currently enabled.
    fn check_initial_state(&self) -> Result<bool, Error>;

    /// Applies the tweak.
    fn apply(&self) -> Result<(), Error>;

    /// Reverts the tweak to its default state.
    fn revert(&self) -> Result<(), Error>;
}

impl TweakAction for Tweak {
    /// Checks the current state of the tweak and updates the `enabled` field accordingly.
    /// This should only be run once when the application starts.
    fn check_initial_state(&self) -> Result<bool, Error> {
        match fetch_tweak_method(self.id) {
            TweakMethod::Registry(method) => {
                return method
                    .is_registry_tweak_enabled(self.id)
                    .map_err(Error::from)
            }
            TweakMethod::GroupPolicy(method) => {
                return method
                    .is_group_policy_tweak_enabled(self.id)
                    .map_err(Error::from)
            }
            TweakMethod::Powershell(method) => {
                return method
                    .is_powershell_script_enabled(self.id)
                    .map_err(Error::from)
            }
        };
    }

    /// Applies the tweak based on its method.
    fn apply(&self) -> Result<(), Error> {
        let result = match fetch_tweak_method(self.id) {
            TweakMethod::Registry(method) => {
                method.apply_registry_tweak(self.id).map_err(Error::from)
            }
            TweakMethod::GroupPolicy(method) => method
                .apply_group_policy_tweak(self.id)
                .map_err(Error::from),
            TweakMethod::Powershell(method) => {
                method.run_apply_script(self.id).map_err(Error::from)
            }
        };

        result
    }

    /// Reverts the tweak to its default state based on its method.
    fn revert(&self) -> Result<(), Error> {
        let result = match fetch_tweak_method(self.id) {
            TweakMethod::Registry(method) => {
                method.revert_registry_tweak(self.id).map_err(Error::from)
            }
            TweakMethod::GroupPolicy(method) => method
                .revert_group_policy_tweak(self.id)
                .map_err(Error::from),
            TweakMethod::Powershell(method) => method.run_undo_script(self.id).map_err(Error::from),
        };

        match &result {
            Ok(_) => {
                tracing::info!(
                    "{:?} -> Tweak reverted successfully, status set to idle.",
                    self.id
                );
            }
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "{:?} -> Failed to revert tweak.", self.id
                );
            }
        }

        result
    }
}
