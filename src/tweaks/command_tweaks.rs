// src/tweaks/command_tweaks.rs

use std::process::Command;

use druid::{im::Vector, Data, Lens};
use once_cell::sync::Lazy;

use super::TweakMethod;
use crate::{models::Tweak, ui::widgets::WidgetType};

#[derive(Clone, Data, Lens, Debug)]
pub struct CommandTweak {
    pub read_commands: Option<Vector<String>>,
    pub apply_commands: Vector<String>,
    pub revert_commands: Option<Vector<String>>,
    pub target_state: Option<Vector<String>>,
}

impl CommandTweak {
    pub fn read_current_state(&self) -> Result<Option<Vec<String>>, anyhow::Error> {
        // For CommandTweak, read can be a no-op or return an appropriate state
        match &self.read_commands {
            Some(commands) => {
                let output = commands.iter().map(|c| {
                    Command::new("cmd")
                        .args(["/C", c])
                        .output()
                        .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", c, e))
                });

                let results: Result<Vec<String>, anyhow::Error> = output
                    .map(|res| {
                        res.and_then(|output| {
                            String::from_utf8(output.stdout).map_err(|e| {
                                anyhow::anyhow!("Failed to convert output to string: {}", e)
                            })
                        })
                    })
                    .collect();
                Ok(Some(results?))
            }
            None => Ok(None),
        }
    }

    pub fn run_apply_script(&self) -> Result<(), anyhow::Error> {
        for script in &self.apply_commands {
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

            // Check if the script indicates that a restart is required
            if stdout.contains("A system restart is required") {
                tracing::debug!("A system restart is required for the changes to take effect.");
                // Update requires_restart flag if necessary
                // Note: You may need to handle state updates appropriately in your application
            }

            tracing::debug!("{}", stdout.trim());
        }
        Ok(())
    }

    pub fn run_revert_script(&self) -> Result<(), anyhow::Error> {
        if let Some(revert_commands) = &self.revert_commands {
            for script in revert_commands {
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
        read_commands: None,
        apply_commands: Vector::from(vec![
            // Run the Process Idle Tasks command
            "Rundll32.exe advapi32.dll,ProcessIdleTasks".to_string(),
        ]),
        revert_commands: None,
        target_state: None,
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
            read_commands: Some(Vector::from(vec![
                // Check if the Ultimate Performance power plan is enabled
                "powercfg /GETACTIVESCHEME".to_string(),
            ])),
            apply_commands: Vector::from(vec![
                // Enable the Ultimate Performance power plan
                "powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61".to_string(),
                // Enable the High Performance power plan
                "powercfg /SETACTIVE bce43009-06f9-424c-a125-20ae96dbec1b".to_string(),
            ]),
            revert_commands: Some(Vector::from(vec![
                // Revert to Balanced plan
                "powercfg -setactive 381b4222-f694-41f0-9685-ff5bb260df2e".to_string(),
                // Optional: Remove the registry key to re-enable Modern Standby
                "powercfg /DELETE bce43009-06f9-424c-a125-20ae96dbec1b".to_string(),
            ])),
            target_state: Some(Vector::from(vec![
                "Power Scheme GUID: bce43009-06f9-424c-a125-20ae96dbec1b  (Ultimate Performance)"
                    .to_string(),
            ])),
        }),
        requires_restart: true,
        applying: false,
    }
});
