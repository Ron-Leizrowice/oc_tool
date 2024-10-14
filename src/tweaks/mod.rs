// src/tweaks/mod.rs

pub mod definitions;
pub mod group_policy;
pub mod powershell;
pub mod registry;

use std::{collections::BTreeMap, sync::Arc};

use anyhow::Error;
use definitions::{
    aggressive_dpc_handling, disable_data_execution_prevention, disable_hpet,
    disable_local_firewall, disable_pagefile, disable_process_idle_states, disable_ram_compression,
    disable_success_auditing, disable_superfetch, kill_all_non_critical_services, kill_explorer,
    low_res_mode, process_idle_tasks,
    registry::{
        additional_kernel_worker_threads, disable_application_telemetry, disable_core_parking,
        disable_driver_paging, disable_hw_acceleration, disable_intel_tsx,
        disable_low_disk_space_checks, disable_page_file_encryption, disable_paging_combining,
        disable_prefetcher, disable_protected_services, disable_security_accounts_manager,
        disable_speculative_execution_mitigations, disable_windows_defender,
        disable_windows_error_reporting, disable_windows_maintenance, dont_verify_random_drivers,
        enable_large_system_cache, enhanced_kernel_performance, high_performance_visual_settings,
        split_large_caches, svc_host_split_threshold, system_responsiveness, thread_dpc_disable,
        win32_priority_separation,
    },
    ultimate_performance_plan,
};
use group_policy::GroupPolicyTweak;
use powershell::PowershellTweak;
use registry::RegistryTweak;

use crate::widgets::TweakWidget;

/// Represents a single tweak that can be applied to the system.
#[derive(Clone)]
pub struct Tweak {
    /// Display name of the tweak.
    pub name: String,
    /// Description of the tweak and its effects, shown in hover tooltip.
    pub description: String,
    /// Category of the tweak, used for grouping tweaks in the UI.
    pub category: TweakCategory,
    /// The method used to apply or revert the tweak.
    pub method: Arc<dyn TweakMethod>,
    /// The widget to use for each tweak
    pub widget: TweakWidget,
    /// Indicates whether the tweak is currently enabled.
    pub enabled: bool,
    /// The status of the tweak (e.g., "Applied", "In Progress", "Failed").
    pub status: TweakStatus,
    /// Whether the tweak requires restarting the system to take effect.
    pub requires_reboot: bool,
    /// If the tweak has been applied during this session, but still requires a reboot.
    pub pending_reboot: bool,
}

/// Trait defining the behavior for all tweak methods.
pub trait TweakMethod: Send + Sync {
    /// Checks if the tweak is currently enabled.
    fn initial_state(&self) -> Result<bool, Error>;

    /// Applies the tweak.
    fn apply(&self) -> Result<(), Error>;

    /// Reverts the tweak.
    fn revert(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweakStatus {
    Idle,
    Applying,
    Reverting,
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakCategory {
    Action,
    System,
    Power,
    Kernel,
    Memory,
    Security,
    Graphics,
    Telemetry,
    Storage,
    Services,
}

impl TweakCategory {
    pub fn left() -> Vec<Self> {
        vec![Self::System, Self::Kernel, Self::Memory, Self::Graphics]
    }

    pub fn right() -> Vec<Self> {
        vec![
            Self::Storage,
            Self::Power,
            Self::Security,
            Self::Telemetry,
            Self::Action,
            Self::Services,
        ]
    }
}

impl Tweak {
    pub fn registry_tweak(
        name: String,
        description: String,
        category: TweakCategory,
        method: RegistryTweak,
        requires_reboot: bool,
    ) -> Self {
        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget: TweakWidget::Toggle,
            requires_reboot,
            status: TweakStatus::Idle,
            enabled: false,
            pending_reboot: false,
        }
    }

    pub fn powershell_tweak(
        name: String,
        description: String,
        category: TweakCategory,
        method: PowershellTweak,
        requires_reboot: bool,
    ) -> Self {
        let widget = match method.undo_script {
            Some(_) => TweakWidget::Toggle,
            None => TweakWidget::Button,
        };

        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget,
            requires_reboot,
            status: TweakStatus::Idle,
            enabled: false,
            pending_reboot: false,
        }
    }

    pub fn group_policy_tweak(
        name: String,
        description: String,
        category: TweakCategory,
        method: GroupPolicyTweak,
        requires_reboot: bool,
    ) -> Self {
        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget: TweakWidget::Toggle,
            requires_reboot,
            status: TweakStatus::Idle,
            enabled: false,
            pending_reboot: false,
        }
    }

    pub fn rust_tweak<M: TweakMethod + 'static>(
        name: String,
        description: String,
        category: TweakCategory,
        method: M,
        widget: TweakWidget,
        requires_reboot: bool,
    ) -> Self {
        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget,
            requires_reboot,
            status: TweakStatus::Idle,
            enabled: false,
            pending_reboot: false,
        }
    }

    pub fn initial_state(&self) -> Result<bool, anyhow::Error> {
        self.method.initial_state()
    }

    pub fn apply(&self) -> Result<(), anyhow::Error> {
        self.method.apply()
    }

    pub fn revert(&self) -> Result<(), anyhow::Error> {
        self.method.revert()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
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
    AdditionalKernelWorkerThreads,
    DisableHPET,
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
    DisableIntelTSX,
    DisableWindowsMaintenance,
    KillExplorer,
    HighPerformanceVisualSettings,
    LowResMode,
    SplitLargeCaches,
    DisableProtectedServices,
    DisableSecurityAccountsManager,
    DisablePagingCombining,
    DisableSuperfetch,
}

/// Initializes all tweaks with their respective configurations.
pub fn all() -> BTreeMap<TweakId, Tweak> {
    BTreeMap::from_iter(vec![
        (TweakId::ProcessIdleTasks, process_idle_tasks()),
        (TweakId::LowResMode, low_res_mode()),
        (TweakId::LargeSystemCache, enable_large_system_cache()),
        (TweakId::SystemResponsiveness, system_responsiveness()),
        (TweakId::DisableHWAcceleration, disable_hw_acceleration()),
        (
            TweakId::Win32PrioritySeparation,
            win32_priority_separation(),
        ),
        (TweakId::DisableCoreParking, disable_core_parking()),
        (
            TweakId::SeLockMemoryPrivilege,
            group_policy::se_lock_memory_privilege(),
        ),
        (
            TweakId::UltimatePerformancePlan,
            ultimate_performance_plan(),
        ),
        (
            TweakId::NoLowDiskSpaceChecks,
            disable_low_disk_space_checks(),
        ),
        (
            TweakId::AdditionalKernelWorkerThreads,
            additional_kernel_worker_threads(),
        ),
        (TweakId::DisableHPET, disable_hpet()),
        (TweakId::AggressiveDpcHandling, aggressive_dpc_handling()),
        (
            TweakId::EnhancedKernelPerformance,
            enhanced_kernel_performance(),
        ),
        (TweakId::DisableRamCompression, disable_ram_compression()),
        (
            TweakId::DisableApplicationTelemetry,
            disable_application_telemetry(),
        ),
        (
            TweakId::DisableWindowsErrorReporting,
            disable_windows_error_reporting(),
        ),
        (TweakId::DisableLocalFirewall, disable_local_firewall()),
        (
            TweakId::DontVerifyRandomDrivers,
            dont_verify_random_drivers(),
        ),
        (TweakId::DisableDriverPaging, disable_driver_paging()),
        (TweakId::DisablePrefetcher, disable_prefetcher()),
        (TweakId::DisableSuccessAuditing, disable_success_auditing()),
        (TweakId::ThreadDpcDisable, thread_dpc_disable()),
        (TweakId::SvcHostSplitThreshold, svc_host_split_threshold()),
        (TweakId::DisablePagefile, disable_pagefile()),
        (
            TweakId::DisableSpeculativeExecutionMitigations,
            disable_speculative_execution_mitigations(),
        ),
        (
            TweakId::DisableDataExecutionPrevention,
            disable_data_execution_prevention(),
        ),
        (TweakId::DisableWindowsDefender, disable_windows_defender()),
        (
            TweakId::DisablePageFileEncryption,
            disable_page_file_encryption(),
        ),
        (
            TweakId::DisableProcessIdleStates,
            disable_process_idle_states(),
        ),
        (
            TweakId::KillAllNonCriticalServices,
            kill_all_non_critical_services(),
        ),
        (TweakId::DisableIntelTSX, disable_intel_tsx()),
        (
            TweakId::DisableWindowsMaintenance,
            disable_windows_maintenance(),
        ),
        (TweakId::KillExplorer, kill_explorer()),
        (
            TweakId::HighPerformanceVisualSettings,
            high_performance_visual_settings(),
        ),
        (TweakId::SplitLargeCaches, split_large_caches()),
        (
            TweakId::DisableProtectedServices,
            disable_protected_services(),
        ),
        (
            TweakId::DisableSecurityAccountsManager,
            disable_security_accounts_manager(),
        ),
        (TweakId::DisablePagingCombining, disable_paging_combining()),
        (TweakId::DisableSuperfetch, disable_superfetch()),
    ])
}
