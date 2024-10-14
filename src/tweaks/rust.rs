// src/tweaks/rust.rs

use super::{
    definitions::{
        kill_explorer::KillExplorerTweak, kill_non_critical_services::KillNonCriticalServicesTweak,
        low_res_mode::LowResMode, process_idle_tasks::ProcessIdleTasksTweak,
    },
    Tweak, TweakCategory, TweakId,
};
use crate::widgets::TweakWidget;

/// Function to create the `Low Resolution Mode` Rust tweak.
pub fn low_res_mode() -> Tweak {
    Tweak::rust_tweak(
        "Low Resolution Mode".to_string(),
        "Sets the display to a lower resolution to conserve resources or improve performance."
            .to_string(),
        TweakCategory::Graphics,
        LowResMode::default(),
        TweakWidget::Toggle,
        false,
    )
}

pub fn process_idle_tasks() -> Tweak {
    Tweak::rust_tweak(
        "Process Idle Tasks".to_string(),
        "Forces the execution of scheduled background tasks that are normally run during system idle time. This helps free up system resources by completing these tasks immediately, improving overall system responsiveness and optimizing resource allocation. It can also reduce latency caused by deferred operations in critical system processes.".to_string(),
        TweakCategory::Action,
        ProcessIdleTasksTweak{
            id: TweakId::ProcessIdleTasks,
        },
        TweakWidget::Button,
        false,
    )
}

pub fn kill_all_non_critical_services() -> Tweak {
    Tweak::rust_tweak(
        "Kill All Non-Critical Services".to_string(),
        "Stops all non-critical services to free up system resources and improve performance. This tweak may cause system instability or data loss.".to_string(),
        TweakCategory::Action,
        KillNonCriticalServicesTweak {
            id: TweakId::KillAllNonCriticalServices,
        },
        TweakWidget::Button,
        false,
    )
}

/// Initializes the Kill Explorer tweak.
pub fn kill_explorer() -> Tweak {
    Tweak::rust_tweak(
        "Kill Explorer".to_string(),
        "Terminates the Windows Explorer process and prevents it from automatically restarting. This can free up system resources but will remove the desktop interface. Use with caution.".to_string(),
        TweakCategory::Action,
        KillExplorerTweak {
            id: TweakId::KillExplorer,
        },
        TweakWidget::Toggle,
        false,
    )
}
