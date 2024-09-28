// src/tweaks/powershell_tweaks.rs

use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use tracing::{debug, error, info, warn};

use super::{Tweak, TweakId, TweakMethod};
use crate::{errors::PowershellError, widgets::TweakWidget};

/// Represents a PowerShell-based tweak, including scripts to read, apply, and undo the tweak.
#[derive(Clone, Debug)]
pub struct PowershellTweak {
    /// PowerShell script to read the current state of the tweak.
    pub read_script: Option<String>,
    /// PowerShell script to apply the tweak.
    pub apply_script: Option<String>,
    /// PowerShell script to undo the tweak.
    pub undo_script: Option<String>,
    /// The target state of the tweak (e.g., the expected output of the read script when the tweak is enabled).
    pub target_state: Option<String>,
}

impl PowershellTweak {
    /// Checks if the tweak is currently enabled by comparing the current value to the default value.
    /// If the current value matches the default value, the tweak is considered enabled.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation succeeds and the tweak is enabled.
    /// - `Ok(false)` if the operation succeeds and the tweak is disabled.
    /// - `Err(anyhow::Error)` if the operation fails.
    pub fn is_powershell_script_enabled(&self, id: TweakId) -> Result<bool, PowershellError> {
        if let Some(target_state) = &self.target_state {
            info!("{:?} -> Checking if PowerShell tweak is enabled.", id);
            match self.read_current_state(id) {
                Ok(Some(current_state)) => {
                    // check if the target state string is contained in the current state
                    let is_enabled = current_state.contains(target_state);
                    debug!(
                        "{:?} -> Current state: {:?}, Target state: {:?}, Enabled: {:?}",
                        id, current_state, target_state, is_enabled
                    );
                    Ok(is_enabled)
                }
                Ok(None) => {
                    warn!(
                        "{:?} -> No read script defined for PowerShell tweak. Assuming disabled.",
                        id
                    );
                    Ok(false)
                }
                Err(e) => {
                    error!(
                        error = ?e,
                        "{:?} -> Failed to read current state of PowerShell tweak.", id
                    );
                    Err(e)
                }
            }
        } else {
            warn!(
                "{:?} -> No target state defined for PowerShell tweak. Assuming disabled.",
                id
            );
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
    pub fn read_current_state(&self, id: TweakId) -> Result<Option<String>, PowershellError> {
        if let Some(script) = &self.read_script {
            info!("{:?} -> Reading current state of PowerShell tweak.", id);
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
                    PowershellError::ScriptExecutionError(format!(
                        "{:?} -> Failed to execute PowerShell script '{:?}': {:?}",
                        id, script, e
                    ))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(
                    "{:?} -> PowerShell script '{:?}' failed with error: {:?}",
                    id,
                    script,
                    stderr.trim()
                );
                return Err(PowershellError::ScriptExecutionError(format!(
                    "PowerShell script '{}' failed with error: {}",
                    script,
                    stderr.trim()
                )));
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("{:?} -> PowerShell script output: {:?}", id, stdout.trim());
            Ok(Some(stdout.trim().to_string()))
        } else {
            debug!(
                "{:?} -> No read script defined for PowerShell tweak. Skipping read operation.",
                id
            );
            Ok(None)
        }
    }

    /// Executes the `apply_script` to apply the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails.
    pub fn run_apply_script(&self, id: TweakId) -> Result<(), PowershellError> {
        match &self.apply_script {
            Some(script) => {
                info!(
                    "{:?} -> Applying PowerShell tweak using script '{:?}'.",
                    id, script
                );
                let output = Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-Command",
                        &script,
                    ])
                    .output()
                    .map_err(|e| {
                        PowershellError::ScriptExecutionError(format!(
                            "{:?} -> Failed to execute PowerShell script '{:?}': {:?}",
                            id, script, e
                        ))
                    })?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    debug!(
                        "{:?} -> Apply script executed successfully. Output: {:?}",
                        id,
                        stdout.trim()
                    );
                    Ok(())
                } else {
                    error!(
                        "{:?} -> PowerShell script '{}' failed with error: {}",
                        id,
                        script,
                        stderr.trim()
                    );
                    return Err(PowershellError::ScriptExecutionError(format!(
                        "PowerShell script '{}' failed with error: {}",
                        script,
                        stderr.trim()
                    )));
                }
            }
            None => {
                warn!(
                    "{:?} -> No apply script defined for PowerShell tweak. Skipping apply operation.",
                    id
                );
                Ok(())
            }
        }
    }

    /// Executes the `undo_script` to revert the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully or no `undo_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    pub fn run_undo_script(&self, id: TweakId) -> Result<(), PowershellError> {
        if let Some(script) = &self.undo_script {
            info!(
                "{:?} -> Reverting PowerShell tweak using script '{:?}'.",
                id, script
            );
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
                    PowershellError::ScriptExecutionError(format!(
                        "{:?} -> Failed to execute PowerShell script '{:?}': {:?}",
                        id, script, e
                    ))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(
                    "{:?} -> PowerShell script '{}' failed with error: {}",
                    id,
                    script,
                    stderr.trim()
                );
                return Err(PowershellError::ScriptExecutionError(format!(
                    "PowerShell script '{}' failed with error: {}",
                    script,
                    stderr.trim()
                )));
            }

            debug!("{:?} -> Revert script executed successfully.", id);
        } else {
            warn!(
                "{:?} -> No undo script defined for PowerShell tweak. Skipping revert operation.",
                id
            );
        }
        Ok(())
    }
}

pub fn process_idle_tasks() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::ProcessIdleTasks,
        "Process Idle Tasks".to_string(),
        "Runs the Process Idle Tasks command to optimize system performance.".to_string(),
        TweakMethod::Powershell(PowershellTweak {
            read_script: None,
            apply_script: Some("Rundll32.exe advapi32.dll,ProcessIdleTasks".to_string()),
            undo_script: None,
            target_state: None,
        }),
        false,
        TweakWidget::Button,
    )
}

pub fn enable_ultimate_performance_plan() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::UltimatePerformancePlan,
        "Enable Ultimate Performance Plan".to_string(),
        "Enables the Ultimate Performance power plan for high-end PCs.".to_string(),
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                powercfg /GETACTIVESCHEME
                "#
                .trim()
                .to_string(),
            ),
            apply_script: Some(
                r#"
                powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61
                $ultimatePlans = powercfg /L | Select-String '(Ultimate Performance)' | ForEach-Object { $_.Line }
                $ultimatePlans = @($ultimatePlans | ForEach-Object { $_ -replace 'Power Scheme GUID: ', '' -replace ' \(Ultimate Performance\)', '' -replace '\*$', '' } | ForEach-Object { $_.Trim() })
                for ($i = 0; $i -lt $ultimatePlans.Length - 1; $i++) { powercfg -delete $ultimatePlans[$i] }
                powercfg /SETACTIVE $ultimatePlans[-1]
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                $balancedPlan = powercfg /L | Select-String '(Balanced)' | ForEach-Object { $_.Line }
                $balancedPlan = $balancedPlan -replace 'Power Scheme GUID: ', '' -replace ' \(Balanced\)', '' -replace '\*$', '' | ForEach-Object { $_.Trim() }
                powercfg /S $balancedPlan
                $ultimatePlans = powercfg /L | Select-String '(Ultimate Performance)' | ForEach-Object { $_.Line }
                $ultimatePlans = @($ultimatePlans | ForEach-Object { $_ -replace 'Power Scheme GUID: ', '' -replace ' \(Ultimate Performance\)', '' -replace '\*$', '' } | ForEach-Object { $_.Trim() })
                foreach ($plan in $ultimatePlans) { powercfg -delete $plan }
                "#
                .trim()
                .to_string(),
            ),
            target_state: Some("(Ultimate Performance)".trim().to_string()),
        }),
        false,
        TweakWidget::Switch,
    )
}
