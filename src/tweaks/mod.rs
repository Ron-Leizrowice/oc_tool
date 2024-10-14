// src/tweaks/mod.rs

pub mod definitions;
pub mod group_policy;
pub mod powershell;
pub mod registry;
pub mod rust;

use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use group_policy::GroupPolicyTweak;
use powershell::PowershellTweak;
use registry::RegistryTweak;

use crate::widgets::TweakWidget;

/// Represents a single tweak that can be applied to the system.
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
}

impl TweakCategory {
    pub fn left() -> Vec<Self> {
        vec![Self::System, Self::Kernel, Self::Memory, Self::Storage]
    }

    pub fn right() -> Vec<Self> {
        vec![
            Self::Graphics,
            Self::Power,
            Self::Security,
            Self::Telemetry,
            Self::Action,
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

    pub fn get_status(&self) -> TweakStatus {
        self.status.clone()
    }

    pub fn set_status(&mut self, status: TweakStatus) {
        self.status = status;
    }

    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_pending_reboot(&mut self, value: bool) {
        self.pending_reboot = value;
    }

    pub fn is_pending_reboot(&self) -> bool {
        self.pending_reboot
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
}

/// Initializes all tweaks with their respective configurations.
pub fn all() -> HashMap<TweakId, Tweak> {
    HashMap::from_iter(vec![
        (TweakId::ProcessIdleTasks, rust::process_idle_tasks()),
        (TweakId::LowResMode, rust::low_res_mode()),
        (
            TweakId::LargeSystemCache,
            definitions::registry::enable_large_system_cache(),
        ),
        (
            TweakId::SystemResponsiveness,
            definitions::registry::system_responsiveness(),
        ),
        (
            TweakId::DisableHWAcceleration,
            definitions::registry::disable_hw_acceleration(),
        ),
        (
            TweakId::Win32PrioritySeparation,
            definitions::registry::win32_priority_separation(),
        ),
        (
            TweakId::DisableCoreParking,
            definitions::registry::disable_core_parking(),
        ),
        (
            TweakId::SeLockMemoryPrivilege,
            group_policy::se_lock_memory_privilege(),
        ),
        (
            TweakId::UltimatePerformancePlan,
            powershell::ultimate_performance_plan(),
        ),
        (
            TweakId::NoLowDiskSpaceChecks,
            definitions::registry::disable_low_disk_space_checks(),
        ),
        (
            TweakId::DisableNtfsTunnelling,
            definitions::registry::disable_ntfs_tunnelling(),
        ),
        (
            TweakId::DistributeTimers,
            definitions::registry::distribute_timers(),
        ),
        (
            TweakId::AdditionalKernelWorkerThreads,
            definitions::registry::additional_kernel_worker_threads(),
        ),
        (TweakId::DisableHPET, powershell::disable_hpet()),
        (
            TweakId::AggressiveDpcHandling,
            powershell::aggressive_dpc_handling(),
        ),
        (
            TweakId::EnhancedKernelPerformance,
            definitions::registry::enhanced_kernel_performance(),
        ),
        (
            TweakId::DisableRamCompression,
            powershell::disable_ram_compression(),
        ),
        (
            TweakId::DisableApplicationTelemetry,
            definitions::registry::disable_application_telemetry(),
        ),
        (
            TweakId::DisableWindowsErrorReporting,
            definitions::registry::disable_windows_error_reporting(),
        ),
        (
            TweakId::DisableLocalFirewall,
            powershell::disable_local_firewall(),
        ),
        (
            TweakId::DontVerifyRandomDrivers,
            definitions::registry::dont_verify_random_drivers(),
        ),
        (
            TweakId::DisableDriverPaging,
            definitions::registry::disable_driver_paging(),
        ),
        (
            TweakId::DisablePrefetcher,
            definitions::registry::disable_prefetcher(),
        ),
        (
            TweakId::DisableSuccessAuditing,
            powershell::disable_success_auditing(),
        ),
        (
            TweakId::ThreadDpcDisable,
            definitions::registry::thread_dpc_disable(),
        ),
        (
            TweakId::SvcHostSplitThreshold,
            definitions::registry::svc_host_split_threshold(),
        ),
        (TweakId::DisablePagefile, powershell::disable_pagefile()),
        (
            TweakId::DisableSpeculativeExecutionMitigations,
            definitions::registry::disable_speculative_execution_mitigations(),
        ),
        (
            TweakId::DisableDataExecutionPrevention,
            powershell::disable_data_execution_prevention(),
        ),
        (
            TweakId::DisableWindowsDefender,
            definitions::registry::disable_windows_defender(),
        ),
        (
            TweakId::DisablePageFileEncryption,
            definitions::registry::disable_page_file_encryption(),
        ),
        (
            TweakId::DisableProcessIdleStates,
            powershell::disable_process_idle_states(),
        ),
        (
            TweakId::KillAllNonCriticalServices,
            rust::kill_all_non_critical_services(),
        ),
        (
            TweakId::DisableIntelTSX,
            definitions::registry::disable_intel_tsx(),
        ),
        (
            TweakId::DisableWindowsMaintenance,
            definitions::registry::disable_windows_maintenance(),
        ),
        (TweakId::KillExplorer, rust::kill_explorer()),
        (
            TweakId::HighPerformanceVisualSettings,
            definitions::registry::high_performance_visual_settings(),
        ),
    ])
}
