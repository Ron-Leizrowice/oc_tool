// src/tweaks/command_tweaks.rs

use druid::im::Vector;
use once_cell::sync::Lazy;

use crate::{CommandTweak, Tweak, TweakMethod, WidgetType};

pub static PROCESS_IDLE_TASKS: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "Process Idle Tasks".to_string(),
    widget: WidgetType::Button,
    enabled: false,
    description: "Runs the Process Idle Tasks command to optimize system performance.".to_string(),
    config: TweakMethod::Command(CommandTweak {
        read_commands: None,
        apply_commands: Vector::from(vec![
            "Rundll32.exe advapi32.dll,ProcessIdleTasks".to_string()
        ]),
        revert_commands: None,
        target_state: None,
    }),
    requires_restart: false,
    applying: false,
});

pub static ENABLE_ULTIMATE_PERFORMANCE_PLAN: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "Enable Ultimate Performance Plan".to_string(),
    enabled: false,
    widget: WidgetType::Switch,
    description: "Enables the Ultimate Performance power plan for high-end PCs.".to_string(),
    config: TweakMethod::Command(CommandTweak {
        read_commands: None,
        apply_commands: Vector::from(vec![
            "powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61".to_string(),
        ]),
        revert_commands: None,
        target_state: None,
    }),
    requires_restart: false,
    applying: false,
});
