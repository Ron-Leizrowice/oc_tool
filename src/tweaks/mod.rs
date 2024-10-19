// src/tweaks/mod.rs

pub mod group_policy;
pub mod msr;
pub mod powershell;
pub mod registry;
pub mod winapi;

use std::{collections::BTreeMap, sync::Arc};

use anyhow::Error;
use group_policy::{all_group_policy_tweaks, method::GroupPolicyTweak};
use msr::{all_msr_tweaks, method::MSRTweak};
use powershell::{all_powershell_tweaks, method::PowershellTweak};
use registry::{all_registry_tweaks, method::RegistryTweak};
use winapi::all_winapi_tweaks;

use crate::widgets::TweakWidget;

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
    SlowMode,
    EnableMcsss,
    DisablePredictiveStoreForwarding,
    DisableSpeculativeStoreBypass,
    DisableSingleThreadIndirectBranchPredictor,
    DisableIndirectBranchRestrictionSpeculation,
    SelectiveBranchPredictorBarrier,
    IndirectBranchPredictionBarrier,
    AutomaticIbrsEnable,
    UpperAddressIgnoreEnable,
    TranslationCacheExtensionEnable,
    FastFxsaveFrstorEnable,
    DisableSecureVirtualMachine,
    DisableNoExecutePage,
    DowngradeFp512ToFp256,
    DisableRsmSpecialBusCycle,
    DisableSmiSpecialBusCycle,
    LongModeEnable,
    SystemCallExtensionEnable,
    AggressivePrefetchProfile,
    DisableUpDownPrefetcher,
    DisableL2StreamPrefetcher,
    DisableL1RegionPrefetcher,
    DisableL1StreamPrefetcher,
    DisableL1StridePrefetcher,
    DisableHostMultiKeyEncryption,
    DisableSecureNestedPaging,
    EnableTopOfMemory2MemoryTypeWriteBack,
    DisableSecureMemoryEncryption,
    EnableMtrrFixedDramAttributes,
    EnableMtrrFixedDramModification,
    EnableMtrrTopOfMemory2,
    EnableMtrrVariableDram,
}

pub fn all_tweaks<'a>() -> BTreeMap<TweakId, Tweak<'a>> {
    let mut tweaks = BTreeMap::new();

    for (id, tweak) in all_registry_tweaks() {
        tweaks.insert(id, tweak);
    }

    for (id, tweak) in all_winapi_tweaks() {
        tweaks.insert(id, tweak);
    }

    for (id, tweak) in all_powershell_tweaks() {
        tweaks.insert(id, tweak);
    }

    for (id, tweak) in all_group_policy_tweaks() {
        tweaks.insert(id, tweak);
    }

    for (id, tweak) in all_msr_tweaks() {
        tweaks.insert(id, tweak);
    }

    tweaks
}

/// Represents a single tweak that can be applied to the system.
pub struct Tweak<'a> {
    /// Display name of the tweak.
    pub name: &'a str,
    /// Description of the tweak and its effects, shown in hover tooltip.
    pub description: &'a str,
    /// Category of the tweak, used for grouping tweaks in the UI.
    pub category: TweakCategory,
    /// The method used to apply or revert the tweak.
    pub method: Arc<dyn TweakMethod>,
    /// The widget to use for each tweak
    pub widget: &'a TweakWidget,
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

#[derive(Debug)]
pub enum TweakStatus {
    Idle,
    Applying,
    Reverting,
    Failed(Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakCategory {
    Action,
    Cpu,
    System,
    Power,
    Kernel,
    Memory,
    Security,
    Graphics,
    Telemetry,
    Services,
}

impl TweakCategory {
    pub fn left() -> Vec<Self> {
        vec![Self::System, Self::Kernel, Self::Memory, Self::Graphics]
    }

    pub fn middle() -> Vec<Self> {
        vec![
            Self::Power,
            Self::Security,
            Self::Telemetry,
            Self::Action,
            Self::Services,
        ]
    }

    pub fn right() -> Vec<Self> {
        vec![Self::Cpu]
    }
}

impl<'a> Tweak<'a> {
    pub fn registry_tweak(
        name: &'a str,
        description: &'a str,
        category: TweakCategory,
        method: RegistryTweak<'static>,
        requires_reboot: bool,
    ) -> Self {
        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget: &TweakWidget::Toggle,
            requires_reboot,
            status: TweakStatus::Idle,
            enabled: false,
            pending_reboot: false,
        }
    }

    pub fn powershell_tweak(
        name: &'a str,
        description: &'a str,
        category: TweakCategory,
        method: PowershellTweak<'static>,
        requires_reboot: bool,
    ) -> Self {
        let widget = match method.undo_script {
            Some(_) => &TweakWidget::Toggle,
            None => &TweakWidget::Button,
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
        name: &'a str,
        description: &'a str,
        category: TweakCategory,
        method: GroupPolicyTweak<'static>,
        requires_reboot: bool,
    ) -> Self {
        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget: &TweakWidget::Toggle,
            requires_reboot,
            status: TweakStatus::Idle,
            enabled: false,
            pending_reboot: false,
        }
    }

    pub fn winapi<M: TweakMethod + 'static>(
        name: &'a str,
        description: &'a str,
        category: TweakCategory,
        method: M,
        widget: &'a TweakWidget,
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

    pub fn msr_tweak(
        name: &'a str,
        description: &'a str,
        category: TweakCategory,
        method: MSRTweak,
        requires_reboot: bool,
    ) -> Self {
        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget: &TweakWidget::Toggle,
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
