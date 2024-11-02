// src/tweaks/powershell.rs

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use tracing::{debug, error, info, warn};

use crate::{
    tweaks::{TweakId, TweakMethod, TweakOption},
    utils::powershell::execute_powershell_script,
};

#[derive(Debug)]
pub struct PowershellTweak<'a> {
    /// The unique ID of the tweak
    pub id: TweakId,
    /// PowerShell script to read the current state of the tweak.
    pub read_script: Option<&'a str>,
    /// Mapping of tweak options to their corresponding PowerShell tweak states.
    pub options: HashMap<TweakOption, &'a PowershellAction<'a>>,
}

#[derive(Debug)]
pub struct PowershellAction<'a> {
    /// PowerShell script to apply or revert the tweak.
    pub script: &'a str,
    /// The target state string to compare against the current state.
    pub state: Option<&'a str>,
}

impl PowershellTweak<'_> {
    /// Reads the current state of the tweak by executing the `read_script`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` with the current state if `read_script` is defined and succeeds.
    /// - `Ok(None)` if no `read_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn read_current_state(&self) -> Result<Option<String>> {
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
    /// Determines the initial state by reading the current state and matching it against defined options.
    ///
    /// # Returns
    /// - `Ok(TweakOption)` representing the current active option.
    /// - `Err(anyhow::Error)` if reading the state fails.
    fn initial_state(&self) -> Result<TweakOption> {
        if let Some(current_state) = self.read_current_state()? {
            info!(
                "{:?} -> Checking current state of PowerShell tweak.",
                self.id
            );
            for (option, tweak_state) in &self.options {
                if let Some(state) = tweak_state.state {
                    if current_state.contains(state) {
                        debug!(
                            "{:?} -> Current state matches option {:?}.",
                            self.id, option
                        );
                        return Ok(option.clone());
                    }
                }
            }
            warn!(
                "{:?} -> Current state does not match any defined option. Assuming default.",
                self.id
            );
            Ok(TweakOption::Enabled(false))
        } else {
            warn!(
                "{:?} -> No read script defined for PowerShell tweak. Assuming default state.",
                self.id
            );
            Ok(TweakOption::Enabled(false))
        }
    }

    /// Applies the specified option by executing its associated script.
    ///
    /// # Arguments
    ///
    /// * `option` - The `TweakOption` to apply.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails or the option is not found.
    fn apply(&self, option: TweakOption) -> Result<()> {
        if let Some(tweak_state) = self.options.get(&option) {
            info!(
                "{:?} -> Applying PowerShell tweak with option {:?} using script '{}'.",
                self.id, option, tweak_state.script
            );

            // Execute the PowerShell script using the custom function
            let output = execute_powershell_script(tweak_state.script).with_context(|| {
                format!(
                    "{:?} -> Failed to execute apply PowerShell script '{}'",
                    self.id, tweak_state.script
                )
            })?;

            debug!(
                "{:?} -> Apply script executed successfully. Output: {}",
                self.id,
                output.trim()
            );
            Ok(())
        } else {
            error!("{:?} -> Option {:?} not found in options.", self.id, option);
            Err(anyhow!(
                "{:?} -> Option {:?} not found in options.",
                self.id,
                option
            ))
        }
    }

    /// Reverts the tweak by applying the default option.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the default script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails or the default option is not defined.
    fn revert(&self) -> Result<()> {
        let default_option = TweakOption::Enabled(false);
        if let Some(default_state) = self.options.get(&default_option) {
            info!(
                "{:?} -> Reverting PowerShell tweak to default using script '{}'.",
                self.id, default_state.script
            );

            // Execute the PowerShell script using the custom function
            let output = execute_powershell_script(default_state.script).with_context(|| {
                format!(
                    "{:?} -> Failed to execute revert PowerShell script '{}'",
                    self.id, default_state.script
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
                "{:?} -> No default option defined for PowerShell tweak. Skipping revert operation.",
                self.id
            );
            Ok(())
        }
    }
}
