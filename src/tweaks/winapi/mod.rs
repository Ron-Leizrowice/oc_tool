// src/tweaks/winapi/mod.rs

pub(crate) mod kill_explorer;
pub(crate) mod kill_non_critical_services;
pub(crate) mod low_res_mode;
pub(crate) mod ultimate_performance_plan;

use kill_explorer::KillExplorerTweak;
use kill_non_critical_services::KillNonCriticalServicesTweak;
use low_res_mode::LowResMode;
use ultimate_performance_plan::UltimatePerformancePlan;

use crate::{
    tweaks::{Tweak, TweakCategory, TweakId},
    ui::TweakWidget,
};

pub fn all_winapi_tweaks<'a>() -> Vec<(TweakId, Tweak<'a>)> {
    vec![
        (
            TweakId::UltimatePerformancePlan,
            ultimate_performance_plan(),
        ),
        (TweakId::LowResMode, low_res_mode()),
        (
            TweakId::KillAllNonCriticalServices,
            kill_all_non_critical_services(),
        ),
        (TweakId::KillExplorer, kill_explorer()),
    ]
}

pub fn ultimate_performance_plan<'a>() -> Tweak<'a> {
    Tweak::winapi(
        "Enable Ultimate Performance Plan",
        "Activates the Ultimate Performance power plan, which is tailored for demanding workloads by minimizing micro-latencies and boosting hardware performance. It disables power-saving features like core parking, hard disk sleep, and processor throttling, ensuring CPU cores run at maximum frequency. This plan also keeps I/O devices and PCIe links at full power, prioritizing performance over energy efficiency. It's designed to reduce the delays introduced by energy-saving policies, improving responsiveness in tasks that require consistent, high-throughput system resources.",
        TweakCategory::Power,
        UltimatePerformancePlan::new(),
        &TweakWidget::Toggle,
        false, // requires reboot
    )
}

pub fn low_res_mode<'a>() -> Tweak<'a> {
    let method = LowResMode::default();

    let formatted_description = format!(
            "Sets the display to lower resolution and refresh rate to reduce GPU load and improve performance -> {}x{} @{}hz.",
            method.target_state.width, method.target_state.height, method.target_state.refresh_rate
        );
    let description: &'a str = Box::leak(formatted_description.into_boxed_str());

    Tweak::winapi(
        "Low Resolution Mode",
        description,
        TweakCategory::Graphics,
        method,
        &TweakWidget::Toggle,
        false,
    )
}

pub fn kill_all_non_critical_services<'a>() -> Tweak<'a> {
    Tweak::winapi(
        "Kill All Non-Critical Services",
        "Stops all non-critical services to free up system resources and improve performance. This tweak may cause system instability or data loss.",
        TweakCategory::Services,
        KillNonCriticalServicesTweak::new(),
        &TweakWidget::Button,
        false,
    )
}

/// Initializes the Kill Explorer tweak.
pub fn kill_explorer<'a>() -> Tweak<'a> {
    Tweak::winapi(
        "Kill Explorer",
        "Terminates the Windows Explorer process and prevents it from automatically restarting. This can free up system resources but will remove the desktop interface. Use with caution.",
        TweakCategory::System,
        KillExplorerTweak::new(),
        &TweakWidget::Toggle,
        false,
    )
}
