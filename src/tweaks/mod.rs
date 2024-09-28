// src/tweaks/mod.rs

pub mod group_policy_tweaks;
pub mod powershell_tweaks;
pub mod registry_tweaks;

use std::{
    hash::Hash,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use anyhow::Error;
use group_policy_tweaks::{se_lock_memory_privilege, GroupPolicyTweak};
use powershell_tweaks::{enable_ultimate_performance_plan, process_idle_tasks, PowershellTweak};
use registry_tweaks::{
    disable_core_parking, disable_hw_acceleration, enable_large_system_cache,
    system_responsiveness, win32_priority_separation, RegistryTweak,
};

use crate::widgets::TweakWidget;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TweakId {
    LargeSystemCache,
    SystemResponsiveness,
    DisableHWAcceleration,
    Win32PrioritySeparation,
    DisableCoreParking,
    ProcessIdleTasks,
    SeLockMemoryPrivilege,
    UltimatePerformancePlan,
}

/// Represents a single tweak that can be applied to the system.
#[derive(Debug, Clone)]
pub struct Tweak {
    /// Unique identifier for the tweak.
    pub id: TweakId,
    pub name: String,
    pub description: String,
    pub method: TweakMethod,
    /// The widget to use for each tweak
    pub widget: TweakWidget,
    /// Indicates whether the tweak is currently enabled.
    pub enabled: Arc<AtomicBool>,
    /// The status of the tweak (e.g., "Applied", "In Progress", "Failed").
    pub status: TweakStatus,
    /// Whether the tweak requires restarting the system to take effect.
    pub requires_reboot: bool,
    /// If the tweak has been applied during this session, but still requires a reboot.
    pub pending_reboot: Arc<AtomicBool>,
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
        requires_reboot: bool,
        widget: TweakWidget,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            id,
            name,
            description,
            method,
            widget,
            enabled: Arc::new(AtomicBool::new(false)),
            status: TweakStatus::Idle,
            requires_reboot,
            pending_reboot: Arc::new(AtomicBool::new(false)),
        }))
    }

    /// Checks if the tweak is currently enabled by invoking the appropriate method.
    pub fn check_initial_state(&self) -> Result<bool, Error> {
        match &self.method {
            TweakMethod::Registry(registry_tweak) => registry_tweak
                .is_registry_tweak_enabled(self.id)
                .map_err(Error::from),
            TweakMethod::GroupPolicy(group_policy_tweak) => group_policy_tweak
                .is_group_policy_tweak_enabled(self.id)
                .map_err(Error::from),
            TweakMethod::Powershell(powershell_tweak) => powershell_tweak
                .is_powershell_script_enabled(self.id)
                .map_err(Error::from),
        }
    }

    /// Applies the tweak by invoking the appropriate method.
    pub fn apply(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.method {
            TweakMethod::Registry(registry_tweak) => {
                registry_tweak.apply_registry_tweak(self.id)?
            }
            TweakMethod::GroupPolicy(group_policy_tweak) => {
                group_policy_tweak.apply_group_policy_tweak(self.id)?
            }
            TweakMethod::Powershell(powershell_tweak) => {
                powershell_tweak.run_apply_script(self.id)?
            }
        }
        Ok(())
    }
    /// Reverts the tweak by invoking the appropriate method.
    pub fn revert(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.method {
            TweakMethod::Registry(registry_tweak) => {
                registry_tweak.revert_registry_tweak(self.id)?
            }
            TweakMethod::GroupPolicy(group_policy_tweak) => {
                group_policy_tweak.revert_group_policy_tweak(self.id)?
            }
            TweakMethod::Powershell(powershell_tweak) => {
                powershell_tweak.run_undo_script(self.id)?
            }
        }
        Ok(())
    }
}

/// Initializes all tweaks with their respective configurations.
pub fn initialize_all_tweaks() -> Vec<Arc<Mutex<Tweak>>> {
    vec![
        enable_large_system_cache(),
        system_responsiveness(),
        disable_hw_acceleration(),
        win32_priority_separation(),
        disable_core_parking(),
        process_idle_tasks(),
        se_lock_memory_privilege(),
        enable_ultimate_performance_plan(),
    ]
}
