// src/tweaks/command_tweaks.rs

use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use super::TweakMethod;
use crate::{
    actions::Tweak,
    tweaks::{add_tweak, TweakId},
};

/// Represents a powershell-based tweak, including scripts to read, apply, and undo the tweak.
#[derive(Clone, Debug)]
pub struct PowershellTweak {
    /// PowerShell script to read the current state of the tweak.
    pub read_script: Option<String>,
    /// PowerShell script to apply the tweak.
    pub apply_script: Option<String>,
    /// PowerShell script to undo the tweak.
    pub undo_script: Option<String>,
    /// The default state to compare against when determining if the tweak is enabled.
    pub default: Option<String>,
}

impl PowershellTweak {
    /// Checks if the tweak is currently enabled by comparing the current value to the default value.
    /// If the current value matches the default value, the tweak is considered enabled.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation succeeds and the tweak is enabled.
    /// - `Ok(false)` if the operation succeeds and the tweak is disabled.
    /// - `Err(anyhow::Error)` if the operation fails.
    pub fn is_powershell_script_enabled(&self) -> Result<bool, anyhow::Error> {
        if let Some(default) = &self.default {
            match self.read_current_state() {
                Ok(Some(current_state)) => Ok(current_state == *default),
                Ok(None) => Ok(false),
                Err(e) => Err(e),
            }
        } else {
            Ok(false)
        }
    }

    /// Reads the current state of the tweak by executing the `read_script`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` with the current state if `read_script` is defined and succeeds.
    /// - `Ok(None)` if no `read_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    pub fn read_current_state(&self) -> Result<Option<String>, anyhow::Error> {
        if let Some(script) = &self.read_script {
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    script,
                ])
                .output()
                .map_err(|e| {
                    anyhow::anyhow!("Failed to execute PowerShell script '{}': {}", script, e)
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

    /// Executes the `apply_script` to apply the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails.
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

    /// Executes the `undo_script` to revert the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully or no `undo_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
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

pub fn initialize_powershell_tweaks() -> Vec<Arc<Mutex<Tweak>>> {
    vec![
        add_tweak(
            TweakId::ProcessIdleTasks,
            "Process Idle Tasks".to_string(),
            "Runs the Process Idle Tasks command to optimize system performance.".to_string(),
             TweakMethod::Powershell(PowershellTweak {
                read_script: None,
                apply_script: Some("Rundll32.exe advapi32.dll,ProcessIdleTasks".to_string()),
                undo_script: None,
                default: None,
            }),
             false,
        ),
        add_tweak(
            TweakId::UltimatePerformancePlan,
            "Enable Ultimate Performance Plan".to_string(),
            "Enables the Ultimate Performance power plan for high-end PCs.".to_string(),
             TweakMethod::Powershell(PowershellTweak {
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

             true,
        ),
    ]
}
