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
use strum_macros::EnumIter;
use winapi::all_winapi_tweaks;

use crate::ui::TweakWidget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, EnumIter)]
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
    DisableRamCompression,
    DisableApplicationTelemetry,
    DisableWindowsErrorReporting,
    DisableTlbCache,
    DisableMcaStatusWriteEnable,
    DisableLocalFirewall,
    DontVerifyRandomDrivers,
    DisableDriverPaging,
    DisablePrefetcher,
    DisableSuccessAuditing,
    ThreadDpcDisable,
    SvcHostSplitThreshold,
    DisablePagefile,
    SpeculativeExecutionMitigations,
    DisableDataExecutionPrevention,
    DisableWindowsDefender,
    DisablePageFileEncryption,
    DisableCpuIdleStates,
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
    EnableMcsss,
    AutomaticIbrsEnable,
    AggressivePrefetchProfile,
    DisableUpDownPrefetcher,
    DisableL2StreamPrefetcher,
    DisableL1RegionPrefetcher,
    DisableL1StreamPrefetcher,
    DisableL1StridePrefetcher,
    EnableMtrrFixedDramAttributes,
    EnableMtrrFixedDramModification,
    EnableTranslationCacheExtension,
    EnableFastFxsaveFrstor,
    DisbleControlFlowEnforcement,
    EnableInterruptibleWbinvd,
    EnableL3CodeDataPrioritization,
    DisableStreamingStores,
    DisableRedirectForReturn,
    DisableOpCache,
    SpeculativeStoreModes,
    DisableAvx512,
    DisableFastShortRepMovsb,
    DisableEnhancedRepMovsbStosb,
    DisableRepMovStosStreaming,
    DisableCoreWatchdogTimer,
    DisablePlatformFirstErrorHandling,
    AlchemyKernelTweak,
    DisableDps
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
    /// The available options for the tweak.
    pub options: Vec<TweakOption>,
    /// Indicates whether the tweak is currently enabled.
    pub state: TweakOption,
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
    fn initial_state(&self) -> Result<TweakOption, Error>;

    /// Applies the tweak.
    fn apply(&self, option: TweakOption) -> Result<(), Error>;

    /// Reverts the tweak.
    fn revert(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum TweakOption {
    Run,
    Enabled(bool),
    Option(String),
}

#[derive(Debug)]
pub enum TweakStatus {
    Idle,
    Busy,
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
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
        vec![Self::Power, Self::Security, Self::Telemetry, Self::Services]
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
        let options: Vec<TweakOption> = method.options.keys().cloned().collect();
        let state = options[0].clone();
        let widget = match state {
            TweakOption::Enabled(_) => &TweakWidget::Toggle,
            TweakOption::Option(_) => &TweakWidget::SettingsComboBox,
            _ => &TweakWidget::Button,
        };

        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            widget,
            options,
            requires_reboot,
            status: TweakStatus::Idle,
            state,
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
        let options: Vec<TweakOption> = method.options.keys().cloned().collect();
        let state = options[0].clone();
        let widget = match state {
            TweakOption::Enabled(_) => &TweakWidget::Toggle,
            TweakOption::Option(_) => &TweakWidget::SettingsComboBox,
            _ => &TweakWidget::Button,
        };

        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            options,
            widget,
            requires_reboot,
            status: TweakStatus::Idle,
            state,
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
        let options: Vec<TweakOption> = method.options.keys().cloned().collect();
        let state = options[0].clone();
        let widget = match state {
            TweakOption::Enabled(_) => &TweakWidget::Toggle,
            TweakOption::Option(_) => &TweakWidget::SettingsComboBox,
            _ => &TweakWidget::Button,
        };

        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            options,
            widget,
            requires_reboot,
            status: TweakStatus::Idle,
            state,
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
            options: vec![TweakOption::Enabled(false), TweakOption::Enabled(true)],
            widget,
            requires_reboot,
            status: TweakStatus::Idle,
            state: TweakOption::Enabled(false),
            pending_reboot: false,
        }
    }

    pub fn msr_tweak(
        name: &'a str,
        description: &'a str,
        category: TweakCategory,
        method: MSRTweak,
    ) -> Self {
        let options: Vec<TweakOption> = method.options.keys().cloned().collect();
        let state = options[0].clone();
        let widget = match state {
            TweakOption::Enabled(_) => &TweakWidget::Toggle,
            TweakOption::Option(_) => &TweakWidget::SettingsComboBox,
            _ => &TweakWidget::Button,
        };
        Self {
            name,
            description,
            category,
            method: Arc::new(method),
            options,
            widget,
            requires_reboot: false,
            status: TweakStatus::Idle,
            state: TweakOption::Enabled(false),
            pending_reboot: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn verify_every_tweak_id_is_used() {
        let all_tweaks = all_tweaks();
        let all_ids = TweakId::iter().collect::<Vec<_>>();

        let mut unused_ids = Vec::new();

        for id in all_ids {
            if !all_tweaks.contains_key(&id) {
                unused_ids.push(id);
            }
        }

        assert!(unused_ids.is_empty(), "Unused IDs: {:?}", unused_ids);
    }
}
