// src/tweaks/mod.rs

pub mod group_policy_tweaks;
pub mod powershell_tweaks;
pub mod registry_tweaks;

use std::sync::{Arc, Mutex};

use group_policy_tweaks::{initialize_group_policy_tweaks, GroupPolicyTweak};
use powershell_tweaks::{initialize_powershell_tweaks, PowershellTweak};
use registry_tweaks::{initialize_registry_tweaks, RegistryTweak};

use crate::{
    actions::Tweak,
    widgets::{button::ApplyButton, switch::ToggleSwitch, TweakWidget},
};

/// Enum representing the method used to apply or revert a tweak.
/// - `Registry`: Modifies Windows Registry keys.
/// - `GroupPolicy`: Adjusts Group Policy settings.
/// - `Command`: Executes PowerShell or other scripts.
#[derive(Clone, Debug)]
pub enum TweakMethod {
    Registry(RegistryTweak),
    GroupPolicy(GroupPolicyTweak),
    Powershell(PowershellTweak),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakId {
    LargeSystemCache,
    SystemResponsiveness,
    DisableHWAcceleration,
    Win32PrioritySeparation,
    DisableLowDiskCheck,
    DisableCoreParking,
    ProcessIdleTasks,
    SeLockMemoryPrivilege,
    UltimatePerformancePlan,
}

pub fn add_tweak(
    id: TweakId,
    name: String,
    description: String,
    method: TweakMethod,
    requires_restart: bool,
) -> Arc<Mutex<Tweak>> {
    let widget = match &method {
        TweakMethod::Registry(_) => TweakWidget::Switch(ToggleSwitch::default()),
        TweakMethod::GroupPolicy(_) => TweakWidget::Switch(ToggleSwitch::default()),
        TweakMethod::Powershell(tweak) => {
            if tweak.undo_script.is_some() {
                TweakWidget::Switch(ToggleSwitch::default())
            } else {
                TweakWidget::Button(ApplyButton::default())
            }
        }
    };

    Arc::new(Mutex::new(Tweak::new(
        id,
        name,
        description,
        method,
        widget,
        requires_restart,
    )))
}

pub fn initialize_all_tweaks() -> Vec<Arc<Mutex<Tweak>>> {
    [
        initialize_powershell_tweaks(),
        initialize_registry_tweaks(),
        initialize_group_policy_tweaks(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
