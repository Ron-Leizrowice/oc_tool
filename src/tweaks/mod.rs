// src/tweaks/mod.rs

pub mod group_policy;
pub mod powershell;
pub mod registry;
pub mod rust;

use std::sync::Arc;

use anyhow::Error;
use dashmap::DashMap;
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
    fn initial_state(&self, id: TweakId) -> Result<bool, Error>;

    /// Applies the tweak.
    fn apply(&self, id: TweakId) -> Result<(), Error>;

    /// Reverts the tweak.
    fn revert(&self, id: TweakId) -> Result<(), Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweakStatus {
    Idle,
    Applying,
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
        vec![
            TweakCategory::System,
            TweakCategory::Kernel,
            TweakCategory::Memory,
            TweakCategory::Storage,
        ]
    }

    pub fn right() -> Vec<Self> {
        vec![
            TweakCategory::Graphics,
            TweakCategory::Power,
            TweakCategory::Security,
            TweakCategory::Telemetry,
            TweakCategory::Action,
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
            widget: TweakWidget::ToggleSwitch,
            requires_reboot,
            status: TweakStatus::Idle,
            enabled: false,
            pending_reboot: false,
        }
    }

    pub fn powershell(
        name: String,
        description: String,
        category: TweakCategory,
        method: PowershellTweak,
        requires_reboot: bool,
    ) -> Self {
        let widget = match method.undo_script {
            Some(_) => TweakWidget::ToggleSwitch,
            None => TweakWidget::ActionButton,
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

    pub fn group_policy(
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
            widget: TweakWidget::ToggleSwitch,
            requires_reboot,
            status: TweakStatus::Idle,
            enabled: false,
            pending_reboot: false,
        }
    }

    pub fn rust<M: TweakMethod + 'static>(
        name: String,
        description: String,
        category: TweakCategory,
        method: M,
        requires_reboot: bool,
    ) -> Self {
        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget: TweakWidget::ToggleSwitch,
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

    pub fn initial_state(&self, id: TweakId) -> Result<bool, anyhow::Error> {
        self.method.initial_state(id)
    }

    pub fn apply(&self, id: TweakId) -> Result<(), anyhow::Error> {
        self.method.apply(id)
    }

    pub fn revert(&self, id: TweakId) -> Result<(), anyhow::Error> {
        self.method.revert(id)
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
pub fn all() -> DashMap<TweakId, Tweak> {
    let tweaks = DashMap::new();

    tweaks.insert(
        TweakId::LargeSystemCache,
        registry::enable_large_system_cache(),
    );
    tweaks.insert(
        TweakId::SystemResponsiveness,
        registry::system_responsiveness(),
    );
    tweaks.insert(
        TweakId::DisableHWAcceleration,
        registry::disable_hw_acceleration(),
    );
    tweaks.insert(
        TweakId::Win32PrioritySeparation,
        registry::win32_priority_separation(),
    );
    tweaks.insert(
        TweakId::DisableCoreParking,
        registry::disable_core_parking(),
    );
    tweaks.insert(TweakId::ProcessIdleTasks, powershell::process_idle_tasks());
    tweaks.insert(
        TweakId::SeLockMemoryPrivilege,
        group_policy::se_lock_memory_privilege(),
    );
    tweaks.insert(
        TweakId::UltimatePerformancePlan,
        powershell::enable_ultimate_performance_plan(),
    );
    tweaks.insert(
        TweakId::NoLowDiskSpaceChecks,
        registry::disable_low_disk_space_checks(),
    );
    tweaks.insert(
        TweakId::DisableNtfsTunnelling,
        registry::disable_ntfs_tunnelling(),
    );
    tweaks.insert(TweakId::DistributeTimers, registry::distribute_timers());
    tweaks.insert(
        TweakId::AdditionalKernelWorkerThreads,
        powershell::additional_kernel_worker_threads(),
    );
    tweaks.insert(TweakId::DisableHPET, powershell::disable_hpet());

    tweaks.insert(
        TweakId::AggressiveDpcHandling,
        powershell::aggressive_dpc_handling(),
    );
    tweaks.insert(
        TweakId::EnhancedKernelPerformance,
        powershell::enhanced_kernel_performance(),
    );
    tweaks.insert(
        TweakId::DisableRamCompression,
        powershell::disable_ram_compression(),
    );
    tweaks.insert(
        TweakId::DisableApplicationTelemetry,
        registry::disable_application_telemetry(),
    );
    tweaks.insert(
        TweakId::DisableWindowsErrorReporting,
        registry::disable_windows_error_reporting(),
    );
    tweaks.insert(
        TweakId::DisableLocalFirewall,
        powershell::disable_local_firewall(),
    );
    tweaks.insert(
        TweakId::DontVerifyRandomDrivers,
        registry::dont_verify_random_drivers(),
    );
    tweaks.insert(
        TweakId::DisableDriverPaging,
        registry::disable_driver_paging(),
    );
    tweaks.insert(TweakId::DisablePrefetcher, registry::disable_prefetcher());
    tweaks.insert(
        TweakId::DisableSuccessAuditing,
        powershell::disable_success_auditing(),
    );
    tweaks.insert(TweakId::ThreadDpcDisable, registry::thread_dpc_disable());
    tweaks.insert(
        TweakId::SvcHostSplitThreshold,
        registry::svc_host_split_threshold(),
    );
    tweaks.insert(TweakId::DisablePagefile, powershell::disable_pagefile());
    tweaks.insert(
        TweakId::DisableSpeculativeExecutionMitigations,
        powershell::disable_speculative_execution_mitigations(),
    );
    tweaks.insert(
        TweakId::DisableDataExecutionPrevention,
        powershell::disable_data_execution_prevention(),
    );
    tweaks.insert(
        TweakId::DisableWindowsDefender,
        registry::disable_windows_defender(),
    );
    tweaks.insert(
        TweakId::DisablePageFileEncryption,
        registry::disable_page_file_encryption(),
    );
    tweaks.insert(
        TweakId::DisableProcessIdleStates,
        powershell::disable_process_idle_states(),
    );
    tweaks.insert(
        TweakId::KillAllNonCriticalServices,
        powershell::kill_all_non_critical_services(),
    );
    tweaks.insert(TweakId::DisableIntelTSX, registry::disable_intel_tsx());
    tweaks.insert(
        TweakId::DisableWindowsMaintenance,
        registry::disable_windows_maintenance(),
    );
    tweaks.insert(TweakId::KillExplorer, powershell::kill_explorer());
    tweaks.insert(
        TweakId::HighPerformanceVisualSettings,
        powershell::high_performance_visual_settings(),
    );
    tweaks.insert(TweakId::LowResMode, rust::low_res_mode());

    tweaks
}
