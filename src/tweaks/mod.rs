// src/tweaks/mod.rs

pub mod group_policy_tweaks;
pub mod powershell_tweaks;
pub mod registry_tweaks;

use std::hash::Hash;

use group_policy_tweaks::{se_lock_memory_privilege, GroupPolicyTweak};
use powershell_tweaks::{enable_ultimate_performance_plan, process_idle_tasks, PowershellTweak};
use registry_tweaks::{
    disable_core_parking, disable_hw_acceleration, enable_large_system_cache,
    system_responsiveness, win32_priority_separation, RegistryTweak,
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

pub fn fetch_tweak_method(id: TweakId) -> TweakMethod {
    match id {
        TweakId::LargeSystemCache => TweakMethod::Registry(enable_large_system_cache()),
        TweakId::SystemResponsiveness => TweakMethod::Registry(system_responsiveness()),
        TweakId::DisableHWAcceleration => TweakMethod::Registry(disable_hw_acceleration()),
        TweakId::Win32PrioritySeparation => TweakMethod::Registry(win32_priority_separation()),
        TweakId::DisableCoreParking => TweakMethod::Registry(disable_core_parking()),
        TweakId::ProcessIdleTasks => TweakMethod::Powershell(process_idle_tasks()),
        TweakId::SeLockMemoryPrivilege => TweakMethod::GroupPolicy(se_lock_memory_privilege()),
        TweakId::UltimatePerformancePlan => {
            TweakMethod::Powershell(enable_ultimate_performance_plan())
        }
    }
}
