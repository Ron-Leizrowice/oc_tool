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
use powershell_tweaks::{
    additional_kernel_worker_threads, aggressive_dpc_handling, disable_data_execution_prevention,
    disable_dynamic_tick, disable_local_firewall, disable_pagefile, disable_process_idle_states,
    disable_ram_compression, disable_speculative_execution_mitigations, disable_success_auditing,
    enable_ultimate_performance_plan, enhanced_kernel_performance, kill_all_non_critical_services,
    process_idle_tasks, PowershellTweak,
};
use registry_tweaks::{
    disable_application_telemetry, disable_core_parking, disable_driver_paging,
    disable_hw_acceleration, disable_low_disk_space_checks, disable_ntfs_tunnelling,
    disable_page_file_encryption, disable_prefetcher, disable_windows_defender,
    disable_windows_error_reporting, distribute_timers, dont_verify_random_drivers,
    enable_large_system_cache, svc_host_split_threshold, system_responsiveness, thread_dpc_disable,
    win32_priority_separation, RegistryTweak,
};

use crate::widgets::TweakWidget;

/// Enum representing the method used to apply or revert a tweak.
/// - `Registry`: Modifies Windows Registry keys.
/// - `GroupPolicy`: Adjusts Group Policy settings.
/// - `Powershell`: Executes PowerShell or other scripts.
#[derive(Clone, Debug)]
pub enum TweakMethod {
    Registry(RegistryTweak),
    GroupPolicy(GroupPolicyTweak),
    Powershell(PowershellTweak),
}

#[derive(Debug, Clone)]
pub enum TweakCategory {
    System,
    Power,
    Kernel,
    Memory,
    Security,
    Graphics,
    Telemetry,
    Storage,
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
    NoLowDiskSpaceChecks,
    DisableNtfsTunnelling,
    DistributeTimers,
    AdditionalKernelWorkerThreads,
    DisableDynamicTick,
    AggressiveDpcHandling,
    EnhancedKernelPerformance,
    DisableRamCompression,
    DisableApplicationTelemetry,
    DisableWindowsErrorReporting,
    DisableLocalFirewall,
    DontVerifyRandomDrivers,
    DisableDriverPaging,
    DisablePrefetcher,
    DisableSuccessAuditing,
    ThreadDpcDisable,
    SvcHostSplitThreshold,
    DisablePagefile,
    DisableSpeculativeExecutionMitigations,
    DisableDataExecutionPrevention,
    DisableWindowsDefender,
    DisablePageFileEncryption,
    DisableProcessIdleStates,
    KillAllNonCriticalServices,
}

/// Represents a single tweak that can be applied to the system.
#[derive(Debug, Clone)]
pub struct Tweak {
    /// Unique identifier for the tweak.
    pub id: TweakId,
    /// Display name of the tweak.
    pub name: String,
    /// Description of the tweak and its effects, shown in hover toolip.
    pub description: String,
    /// Category of the tweak, used for grouping tweaks in the UI.
    pub category: TweakCategory,
    /// List of citations for the tweak, shown in the tweak details.
    pub citations: Vec<String>,
    /// The method used to apply or revert the tweak.
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
        category: TweakCategory,
        citations: Vec<String>,
        method: TweakMethod,
        requires_reboot: bool,
        widget: TweakWidget,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            id,
            name,
            description,
            category,
            citations,
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
        disable_low_disk_space_checks(),
        disable_ntfs_tunnelling(),
        distribute_timers(),
        additional_kernel_worker_threads(),
        disable_dynamic_tick(),
        aggressive_dpc_handling(),
        enhanced_kernel_performance(),
        disable_ram_compression(),
        disable_application_telemetry(),
        disable_windows_error_reporting(),
        disable_local_firewall(),
        dont_verify_random_drivers(),
        disable_driver_paging(),
        disable_prefetcher(),
        disable_success_auditing(),
        thread_dpc_disable(),
        svc_host_split_threshold(),
        disable_pagefile(),
        disable_speculative_execution_mitigations(),
        disable_data_execution_prevention(),
        disable_windows_defender(),
        disable_page_file_encryption(),
        disable_process_idle_states(),
        kill_all_non_critical_services(),
    ]
}
