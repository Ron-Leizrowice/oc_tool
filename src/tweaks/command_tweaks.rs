// src/tweaks/command_tweaks.rs

use std::process::Command;

use druid::{Data, Lens};
use once_cell::sync::Lazy;

use super::TweakMethod;
use crate::{models::Tweak, ui::widgets::WidgetType};

#[derive(Clone, Data, Lens, Debug)]
pub struct CommandTweak {
    pub read_script: Option<String>,
    pub apply_script: Option<String>,
    pub undo_script: Option<String>,
    pub default: Option<String>,
}

impl CommandTweak {
    pub fn read_current_state(&self) -> Result<Option<String>, anyhow::Error> {
        if let Some(command) = &self.read_script {
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    command,
                ])
                .output()
                .map_err(|e| {
                    anyhow::anyhow!("Failed to execute PowerShell script '{}': {}", command, e)
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!(
                    "PowerShell script failed with error: {}",
                    stderr.trim()
                ));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(Some(stdout.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn run_apply_script(&self) -> Result<(), anyhow::Error> {
        if let Some(script) = &self.apply_script {
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    script,
                ])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to execute PowerShell script: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "PowerShell script failed with error: {}",
                    stderr.trim()
                ));
            }

            tracing::debug!("{}", stdout.trim());
        }
        Ok(())
    }

    pub fn run_undo_script(&self) -> Result<(), anyhow::Error> {
        if let Some(script) = &self.undo_script {
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    script,
                ])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to execute PowerShell script: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!(
                    "PowerShell script failed with error: {}",
                    stderr.trim()
                ));
            }
        }
        Ok(())
    }
}

pub static PROCESS_IDLE_TASKS: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "Process Idle Tasks".to_string(),
    description: "Runs the Process Idle Tasks command to optimize system performance.".to_string(),
    widget: WidgetType::Button,
    enabled: false,
    method: TweakMethod::Command(CommandTweak {
        read_script: None,
        apply_script: Some("Rundll32.exe advapi32.dll,ProcessIdleTasks".to_string()),
        undo_script: None,
        default: None,
    }),
    requires_restart: false,
    applying: false,
});

pub static ENABLE_ULTIMATE_PERFORMANCE_PLAN: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
        id: 0,
        name: "Enable Ultimate Performance Plan".to_string(),
        description: "Enables the Ultimate Performance power plan for high-end PCs.".to_string(),
        enabled: false,
        widget: WidgetType::Switch,
        method: TweakMethod::Command(CommandTweak {
            read_script: Some("powercfg /GETACTIVESCHEME".to_string()),
            apply_script: Some(
                "powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61; powercfg /SETACTIVE bce43009-06f9-424c-a125-20ae96dbec1b".to_string(),
            ),
            undo_script: Some(
                "powercfg -setactive 381b4222-f694-41f0-9685-ff5bb260df2e; powercfg /DELETE bce43009-06f9-424c-a125-20ae96dbec1b".to_string(),
            ),
            default: Some(
                "Power Scheme GUID: bce43009-06f9-424c-a125-20ae96dbec1b  (Ultimate Performance)".to_string(),
            ),
        }),
        requires_restart: true,
        applying: false,
    }
});
