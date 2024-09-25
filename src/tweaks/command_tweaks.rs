// src/tweaks/command_tweaks.rs

use std::process::Command;

use druid::{im::Vector, Data, Lens};
use once_cell::sync::Lazy;

use super::TweakMethod;
use crate::{actions::TweakAction, models::Tweak, ui::widgets::WidgetType};

#[derive(Clone, Data, Lens, Debug)]
pub struct CommandTweak {
    pub read_commands: Option<Vector<String>>,
    pub apply_commands: Vector<String>,
    pub revert_commands: Option<Vector<String>>,
    pub target_state: Option<Vector<String>>,
}

impl CommandTweak {
    pub fn is_enabled(&self) -> bool {
        // For CommandTweaks, attempt to read the current state, and compare with the default state
        match self.target_state {
            Some(ref target_state) => {
                let current_state = self.read_current_state().unwrap_or(None);
                current_state == Some(target_state.clone().into_iter().collect())
            }
            None => false,
        }
    }

    pub fn read_current_state(&self) -> Result<Option<Vec<String>>, anyhow::Error> {
        // For CommandTweak, read can be a no-op or return an appropriate state
        match &self.read_commands {
            Some(commands) => {
                let output = commands.iter().map(|c| {
                    Command::new("cmd")
                        .args(&["/C", c])
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

    pub fn apply(&self) -> Result<(), anyhow::Error> {
        let result = self.apply_commands.iter().try_for_each(|c| {
            let output = Command::new("cmd")
                .args(&["/C", c])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", c, e))?;

            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!(
                    "Command '{}' failed with error: {}",
                    c,
                    stderr
                ))
            }
        });

        match result {
            Ok(_) => {
                println!("Successfully applied the tweak");
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to apply the tweak: {}", e);
                Err(e)
            }
        }
    }

    pub fn revert(&self) -> Result<(), anyhow::Error> {
        if let Some(revert_commands) = &self.revert_commands {
            revert_commands.iter().try_for_each(|c| {
                let output = Command::new("cmd")
                    .args(&["/C", c])
                    .output()
                    .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", c, e))?;

                if output.status.success() {
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(anyhow::anyhow!(
                        "Command '{}' failed with error: {}",
                        c,
                        stderr
                    ))
                }
            })
        } else {
            Ok(())
        }
    }
}

impl TweakAction for CommandTweak {
    fn read(&self) -> Result<(), anyhow::Error> {
        if let Some(target_state) = &self.target_state {
            let current_state = self.read_current_state()?;
            if current_state != Some(target_state.clone().into_iter().collect()) {
                return Err(anyhow::anyhow!("Current state does not match target state"));
            }
        }
        Ok(())
    }

    fn apply(&self) -> Result<(), anyhow::Error> {
        self.apply()
    }

    fn revert(&self) -> Result<(), anyhow::Error> {
        self.revert()
    }
}

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
