// src/tweaks/powershell.rs

use anyhow::Context;
use tracing::{debug, error, info, warn};

use crate::{
    tweaks::{TweakId, TweakMethod},
    utils::powershell::execute_powershell_script,
};

/// Represents a PowerShell-based tweak, including scripts to read, apply, and undo the tweak.
#[derive(Clone, Debug)]
pub struct PowershellTweak<'a> {
    /// The unique ID of the tweak
    pub id: TweakId,
    /// PowerShell script to read the current state of the tweak.
    pub read_script: Option<&'a str>,
    /// PowerShell script to apply the tweak.
    pub apply_script: &'a str,
    /// PowerShell script to undo the tweak.
    pub undo_script: Option<&'a str>,
    /// The target state of the tweak (e.g., the expected output of the read script when the tweak is enabled).
    pub target_state: Option<&'a str>,
}

impl PowershellTweak<'_> {
    /// Reads the current state of the tweak by executing the `read_script`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` with the current state if `read_script` is defined and succeeds.
    /// - `Ok(None)` if no `read_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn read_current_state(&self) -> anyhow::Result<Option<String>> {
        if let Some(script) = &self.read_script {
            info!(
                "{:?} -> Reading current state of PowerShell tweak.",
                self.id
            );

            // Execute the PowerShell script using the custom function
            let output = execute_powershell_script(script).with_context(|| {
                format!(
                    "{:?} -> Failed to execute read PowerShell script '{}'",
                    self.id, script
                )
            })?;

            debug!(
                "{:?} -> PowerShell script output: {}",
                self.id,
                output.trim()
            );

            Ok(Some(output.trim().to_string()))
        } else {
            debug!(
                "{:?} -> No read script defined for PowerShell tweak. Skipping read operation.",
                self.id
            );
            Ok(None)
        }
    }
}

impl TweakMethod for PowershellTweak<'_> {
    /// Checks if the tweak is currently enabled by comparing the current value to the default value.
    /// If the current value matches the default value, the tweak is considered enabled.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation succeeds and the tweak is enabled.
    /// - `Ok(false)` if the operation succeeds and the tweak is disabled.
    /// - `Err(anyhow::Error)` if the operation fails.
    fn initial_state(&self) -> anyhow::Result<bool> {
        if let Some(target_state) = &self.target_state {
            info!("{:?} -> Checking if PowerShell tweak is enabled.", self.id);
            match self.read_current_state() {
                Ok(Some(current_state)) => {
                    // check if the target state string is contained in the current state
                    let is_enabled = current_state.contains(target_state);
                    debug!(
                        "{:?} -> Current state: '{}', Target state: '{}', Enabled: {}",
                        self.id, current_state, target_state, is_enabled
                    );
                    Ok(is_enabled)
                }
                Ok(None) => {
                    warn!(
                        "{:?} -> No read script defined for PowerShell tweak. Assuming disabled.",
                        self.id
                    );
                    Ok(false)
                }
                Err(e) => {
                    error!(
                        error = ?e,
                        "{:?} -> Failed to read current state of PowerShell tweak.", self.id
                    );
                    Err(e)
                }
            }
        } else {
            warn!(
                "{:?} -> No target state defined for PowerShell tweak. Assuming disabled.",
                self.id
            );
            Ok(false)
        }
    }

    /// Executes the `apply_script` to apply the tweak synchronously.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn apply(&self) -> anyhow::Result<()> {
        info!(
            "{:?} -> Applying PowerShell tweak using script '{}'.",
            self.id, &self.apply_script
        );

        // Execute the PowerShell script using the custom function
        let output = execute_powershell_script(self.apply_script).with_context(|| {
            format!(
                "{:?} -> Failed to execute apply PowerShell script '{}'",
                self.id, &self.apply_script
            )
        })?;

        debug!(
            "{:?} -> Apply script executed successfully. Output: {}",
            self.id,
            output.trim()
        );
        Ok(())
    }

    /// Executes the `undo_script` to revert the tweak synchronously.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully or no `undo_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn revert(&self) -> anyhow::Result<()> {
        if let Some(script) = &self.undo_script {
            info!(
                "{:?} -> Reverting PowerShell tweak using script '{}'.",
                self.id, script
            );

            // Execute the PowerShell script using the custom function
            let output = execute_powershell_script(script).with_context(|| {
                format!(
                    "{:?} -> Failed to execute revert PowerShell script '{}'",
                    self.id, script
                )
            })?;

            debug!(
                "{:?} -> Revert script executed successfully. Output: {}",
                self.id,
                output.trim()
            );
            Ok(())
        } else {
            warn!(
                "{:?} -> No undo script defined for PowerShell tweak. Skipping revert operation.",
                self.id
            );
            Ok(())
        }
    }
}
