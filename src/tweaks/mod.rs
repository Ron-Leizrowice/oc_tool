// src/tweaks/mod.rs

pub mod definitions;
pub mod group_policy;
pub mod msr;
pub mod powershell;
pub mod registry;

use std::sync::Arc;

use anyhow::Error;
use group_policy::GroupPolicyTweak;
use msr::MSRTweak;
use powershell::PowershellTweak;
use registry::RegistryTweak;

use crate::widgets::TweakWidget;

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

    pub fn right() -> Vec<Self> {
        vec![
            Self::Power,
            Self::Security,
            Self::Telemetry,
            Self::Action,
            Self::Services,
        ]
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

    pub fn rust_tweak<M: TweakMethod + 'static>(
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
