// src/tweaks/mod.rs
pub mod group_policy_tweaks;
pub mod method;
pub mod powershell_tweaks;
pub mod registry_tweaks;
pub mod rust_tweaks;

use std::{
    hash::Hash,
    sync::{Arc, Mutex},
};

use group_policy_tweaks::GroupPolicyTweak;
use method::TweakMethod;
use powershell_tweaks::PowershellTweak;
use registry_tweaks::RegistryTweak;

use crate::widgets::TweakWidget;

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

/// Represents a single tweak that can be applied to the system.
#[derive(Clone)]
pub struct Tweak {
    /// Unique identifier for the tweak.
    pub id: TweakId,
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
    pub enabled: Arc<Mutex<bool>>,
    /// The status of the tweak (e.g., "Applied", "In Progress", "Failed").
    pub status: Arc<Mutex<TweakStatus>>,
    /// Whether the tweak requires restarting the system to take effect.
    pub requires_reboot: bool,
    /// If the tweak has been applied during this session, but still requires a reboot.
    pub pending_reboot: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweakStatus {
    Idle,
    Applying,
    Failed(String),
}

impl Tweak {
    pub fn registry_tweak(
        id: TweakId,
        name: String,
        description: String,
        category: TweakCategory,
        method: RegistryTweak,
        requires_reboot: bool,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            id,
            name,
            description,
            category,
            method: Arc::new(method),
            widget: TweakWidget::ToggleSwitch,
            enabled: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new(TweakStatus::Idle)),
            requires_reboot,
            pending_reboot: Arc::new(Mutex::new(false)),
        }))
    }

    pub fn powershell(
        id: TweakId,
        name: String,
        description: String,
        category: TweakCategory,
        method: PowershellTweak,
        requires_reboot: bool,
    ) -> Arc<Mutex<Self>> {
        let widget = match method.undo_script {
            Some(_) => TweakWidget::ToggleSwitch,
            None => TweakWidget::ActionButton,
        };

        Arc::new(Mutex::new(Self {
            id,
            name,
            description,
            category,
            method: Arc::new(method),
            widget,
            enabled: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new(TweakStatus::Idle)),
            requires_reboot,
            pending_reboot: Arc::new(Mutex::new(false)),
        }))
    }

    pub fn group_policy(
        id: TweakId,
        name: String,
        description: String,
        category: TweakCategory,
        method: GroupPolicyTweak,
        requires_reboot: bool,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            id,
            name,
            description,
            category,
            method: Arc::new(method),
            widget: TweakWidget::ToggleSwitch,
            enabled: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new(TweakStatus::Idle)),
            requires_reboot,
            pending_reboot: Arc::new(Mutex::new(false)),
        }))
    }

    pub fn rust<M: TweakMethod + 'static>(
        id: TweakId,
        name: String,
        description: String,
        category: TweakCategory,
        method: M,
        requires_reboot: bool,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            id,
            name,
            description,
            category,
            method: Arc::new(method),
            widget: TweakWidget::ToggleSwitch,
            enabled: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new(TweakStatus::Idle)),
            requires_reboot,
            pending_reboot: Arc::new(Mutex::new(false)),
        }))
    }

    pub fn get_status(&self) -> TweakStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn set_status(&self, status: TweakStatus) {
        *self.status.lock().unwrap() = status;
    }

    pub fn set_enabled(&self) {
        *self.enabled.lock().unwrap() = true;
    }

    pub fn set_disabled(&self) {
        *self.enabled.lock().unwrap() = false;
    }

    pub fn pending_reboot(&self) {
        *self.pending_reboot.lock().unwrap() = true;
    }

    pub fn cancel_pending_reboot(&self) {
        *self.pending_reboot.lock().unwrap() = false;
    }

    pub fn initial_state(&self) -> Result<bool, anyhow::Error> {
        self.method.initial_state(self.id)
    }

    pub fn apply(&self) -> Result<(), anyhow::Error> {
        self.method.apply(self.id)
    }

    pub fn revert(&self) -> Result<(), anyhow::Error> {
        self.method.revert(self.id)
    }
}

/// Initializes all tweaks with their respective configurations.
pub fn tweak_list() -> Vec<Arc<Mutex<Tweak>>> {
    vec![
        rust_tweaks::low_res_mode(),
        group_policy_tweaks::se_lock_memory_privilege(),
        powershell_tweaks::process_idle_tasks(),
        powershell_tweaks::enable_ultimate_performance_plan(),
        powershell_tweaks::additional_kernel_worker_threads(),
        powershell_tweaks::disable_hpet(),
        powershell_tweaks::aggressive_dpc_handling(),
        powershell_tweaks::enhanced_kernel_performance(),
        powershell_tweaks::disable_ram_compression(),
        powershell_tweaks::disable_pagefile(),
        powershell_tweaks::disable_speculative_execution_mitigations(),
        powershell_tweaks::disable_data_execution_prevention(),
        powershell_tweaks::kill_explorer(),
        powershell_tweaks::high_performance_visual_settings(),
        powershell_tweaks::disable_local_firewall(),
        powershell_tweaks::disable_process_idle_states(),
        powershell_tweaks::kill_all_non_critical_services(),
        powershell_tweaks::disable_success_auditing(),
        registry_tweaks::enable_large_system_cache(),
        registry_tweaks::system_responsiveness(),
        registry_tweaks::disable_hw_acceleration(),
        registry_tweaks::win32_priority_separation(),
        registry_tweaks::disable_core_parking(),
        registry_tweaks::disable_low_disk_space_checks(),
        registry_tweaks::disable_ntfs_tunnelling(),
        registry_tweaks::distribute_timers(),
        registry_tweaks::disable_application_telemetry(),
        registry_tweaks::disable_windows_error_reporting(),
        registry_tweaks::dont_verify_random_drivers(),
        registry_tweaks::disable_driver_paging(),
        registry_tweaks::disable_prefetcher(),
        registry_tweaks::thread_dpc_disable(),
        registry_tweaks::svc_host_split_threshold(),
        registry_tweaks::disable_windows_defender(),
        registry_tweaks::disable_page_file_encryption(),
        registry_tweaks::disable_intel_tsx(),
        registry_tweaks::disable_windows_maintenance(),
    ]
}
