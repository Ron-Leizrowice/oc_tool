// src/tweaks/registry.rs

use anyhow::{Context, Result};
use indexmap::IndexMap;
use tracing::{debug, error, trace};

use crate::{
    tweaks::{TweakId, TweakMethod, TweakOption},
    utils::registry::{
        create_or_modify_registry_value, delete_registry_value, read_registry_value,
        RegistryKeyValue,
    },
};

/// Represents a single registry modification, including the registry key, value name,
/// and the desired value when enabled.
#[derive(Debug, Clone)]
pub struct RegistryModification<'a> {
    /// Full path of the registry key (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
    pub path: &'a str,
    /// Name of the registry value to modify.
    pub key: &'a str,
    /// The value to set when the tweak is enabled.
    pub value: RegistryKeyValue,
}

/// Defines a set of modifications to the Windows registry, which in combination
/// make up a single tweak.
#[derive(Debug, Clone)]
pub struct RegistryTweak<'a> {
    /// Unique ID for the tweak
    pub id: TweakId,
    /// Mapping from TweakOption to a list of registry modifications.
    pub options: IndexMap<TweakOption, Vec<RegistryModification<'a>>>,
}

impl RegistryTweak<'_> {
    /// Reads the current values of all registry modifications for a given option.
    ///
    ///
    /// # Parameters
    ///
    /// - `option`: The `TweakOption` to read values for.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if all the values were read and match the expected values.
    /// - `Ok(false)` if any value was not found or did not match the expected value.
    /// - `Err(anyhow::Error)` if any operation fails.
    pub fn read_current_values(&self, option: &TweakOption) -> Result<bool> {
        trace!(
            "{:?} -> Reading current values of registry tweak for option: {:?}",
            self.id,
            option
        );
        let modifications = self.options.get(option).context(format!(
            "{:?} -> No registry modifications found for option: {:?}",
            self.id, option
        ))?;

        for modification in modifications {
            let current_value =
                read_registry_value(modification.path, modification.key).context(format!(
                    "{:?} -> Failed to read value '{}' from '{}' for option: {:?}",
                    self.id, modification.key, modification.path, option
                ))?;

            match &modification.value {
                RegistryKeyValue::Deleted => {
                    if current_value.is_some() {
                        debug!(
                            "{:?} -> Value '{}' found in '{}' for option: {:?}",
                            self.id, modification.key, modification.path, option
                        );
                        return Ok(false);
                    }
                }
                _ => {
                    if let Some(current) = current_value {
                        if modification.value != current {
                            debug!(
                                "{:?} -> Value '{}' in '{}' does not match expected value: {:?}",
                                self.id, modification.key, modification.path, modification.value
                            );
                            return Ok(false);
                        }
                    } else {
                        debug!(
                            "{:?} -> Value '{}' not found in '{}' for option: {:?}",
                            self.id, modification.key, modification.path, option
                        );
                        return Ok(false);
                    }
                }
            }
        }

        trace!(
            "{:?} -> All values match expected values for option: {:?}",
            self.id,
            option
        );

        Ok(true)
    }

    /// Rolls back modifications by applying the `Default` option's modifications.
    ///
    /// # Parameters
    ///
    /// - `operation`: A string indicating the operation ("apply" or "revert") for logging purposes.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all rollback operations succeed.
    /// - `Err(anyhow::Error)` if any rollback operation fails.
    fn rollback(&self, operation: &str) -> Result<()> {
        debug!(
            "{:?} -> Initiating rollback for {} operation by applying Default modifications.",
            self.id, operation
        );

        let default_modifications =
            self.options
                .get(&TweakOption::Enabled(false))
                .context(format!(
                    "{:?} -> No registry modifications found for Default option during rollback.",
                    self.id
                ))?;

        for modification in default_modifications {
            match &modification.value {
                RegistryKeyValue::Deleted => {
                    // Delete the registry value
                    delete_registry_value(modification.path, modification.key).with_context(
                        || {
                            format!(
                                "{:?} -> Failed to delete value '{}' in '{}' during rollback.",
                                self.id, modification.key, modification.path
                            )
                        },
                    )?;
                    debug!(
                        "{:?} -> Deleted value '{}' in '{}'.",
                        self.id, modification.key, modification.path
                    );
                }
                _ => {
                    // Apply the default value using the helper function
                    create_or_modify_registry_value(
                        modification.path,
                        modification.key,
                        &modification.value,
                    )
                    .with_context(|| {
                        format!(
                            "{:?} -> Failed to set default value '{}' in '{}' during rollback.",
                            self.id, modification.key, modification.path
                        )
                    })?;
                    debug!(
                        "{:?} -> Set default value '{}' to {:?} in '{}'.",
                        self.id, modification.key, modification.value, modification.path
                    );
                }
            }
        }

        debug!(
            "{:?} -> Successfully rolled back {} operation.",
            self.id, operation
        );
        Ok(())
    }
}

impl TweakMethod for RegistryTweak<'_> {
    /// Checks the current state of the registry tweak and returns the corresponding `TweakOption`.
    ///
    /// # Returns
    /// - `Ok(TweakOption)` indicating the current state.
    /// - `Err(anyhow::Error)` if the operation fails.
    fn initial_state(&self) -> Result<TweakOption, anyhow::Error> {
        debug!(
            "{:?} -> Determining the initial state of the registry tweak.",
            self.id
        );

        // Iterate through all possible options to find which one matches the current state
        for (option, modifications) in &self.options {
            let mut all_match = true;
            for modification in modifications {
                match &modification.value {
                    RegistryKeyValue::Deleted => {
                        // For Deleted, ensure the value does not exist
                        let exists =
                            read_registry_value(modification.path, modification.key)?.is_some();
                        if exists {
                            all_match = false;
                            break;
                        }
                    }
                    _ => {
                        // For other types, compare the current value with the expected value
                        let current = read_registry_value(modification.path, modification.key)?;
                        if current != Some(modification.value.clone()) {
                            all_match = false;
                            break;
                        }
                    }
                }
            }
            if all_match {
                tracing::debug!("{:?} -> Current state matches {:?}.", self.id, option);
                return Ok(option.clone());
            }
        }

        // If no matching option is found, consider it as Default
        tracing::debug!(
            "{:?} -> Current state does not match any custom options. Reverting to Default.",
            self.id
        );
        Ok(self
            .options
            .keys()
            .next()
            .context("No keys found in options")?
            .clone())
    }

    /// Applies the registry tweak based on the selected `TweakOption`.
    ///
    /// # Parameters
    ///
    /// - `option`: The `TweakOption` to apply (Default or Custom).
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations succeed.
    /// - `Err(anyhow::Error)` if any operation fails, after attempting rollback.
    fn apply(&self, option: TweakOption) -> Result<(), anyhow::Error> {
        debug!(
            "Applying registry tweak '{:?}' with option: {:?}.",
            self.id, option
        );
        let modifications = self.options.get(&option).context(format!(
            "{:?} -> No registry modifications found for option: {:?}",
            self.id, option
        ))?;
        let mut applied_modifications = Vec::new();

        // Wrap the apply logic in a closure to handle errors and perform rollback
        let result: Result<(), anyhow::Error> = (|| -> Result<(), anyhow::Error> {
            for modification in modifications {
                match &modification.value {
                    RegistryKeyValue::Deleted => {
                        // Delete the registry value
                        delete_registry_value(modification.path, modification.key).with_context(
                            || {
                                format!(
                                    "{:?} -> Failed to delete value '{}' in '{}'",
                                    self.id, modification.key, modification.path
                                )
                            },
                        )?;
                        debug!(
                            "{:?} -> Deleted value '{}' in '{}'.",
                            self.id, modification.key, modification.path
                        );
                    }
                    _ => {
                        // Apply the tweak value using the helper function
                        create_or_modify_registry_value(
                            modification.path,
                            modification.key,
                            &modification.value,
                        )
                        .with_context(|| {
                            format!(
                                "{:?} -> Failed to set value '{}' in '{}'",
                                self.id, modification.key, modification.path
                            )
                        })?;
                        debug!(
                            "{:?} -> Set value '{}' to {:?} in '{}'.",
                            self.id, modification.key, modification.value, modification.path
                        );
                    }
                }

                // Record the successfully applied modification
                applied_modifications.push(modification.clone());
            }
            Ok(())
        })();

        if let Err(e) = result {
            // An error occurred during apply
            error!(
                "{:?} -> Error occurred during apply: {}. Attempting to rollback applied modifications.",
                self.id, e
            );

            // Attempt to rollback by applying Default modifications
            if let Err(rollback_err) = self.rollback("apply") {
                // Rollback failed
                error!(
                    "{:?} -> Failed to rollback after apply error: {}",
                    self.id, rollback_err
                );
                // Return an error indicating both the original error and the rollback error
                anyhow::bail!("Apply failed: {}. Rollback failed: {}", e, rollback_err);
            } else {
                debug!(
                    "{:?} -> Successfully rolled back after apply error.",
                    self.id
                );
            }
            // Return the original error
            return Err(e);
        }

        debug!(
            "{:?} -> Successfully applied registry tweak with option: {:?}.",
            self.id, option
        );
        Ok(())
    }

    /// Reverts the registry tweak by applying the `Default` option's modifications.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations succeed.
    /// - `Err(anyhow::Error)` if any operation fails, after attempting rollback.
    fn revert(&self) -> Result<(), anyhow::Error> {
        debug!("{:?} -> Reverting registry tweak to Default.", self.id);
        let default_modifications =
            self.options
                .get(&TweakOption::Enabled(false))
                .context(format!(
                    "{:?} -> No registry modifications found for Default option.",
                    self.id
                ))?;

        // Apply each default modification
        for modification in default_modifications {
            match &modification.value {
                RegistryKeyValue::Deleted => {
                    // Delete the registry value
                    delete_registry_value(modification.path, modification.key).with_context(
                        || {
                            format!(
                                "{:?} -> Failed to delete value '{}' in '{}'",
                                self.id, modification.key, modification.path
                            )
                        },
                    )?;
                    debug!(
                        "{:?} -> Deleted value '{}' in '{}'.",
                        self.id, modification.key, modification.path
                    );
                }
                _ => {
                    // Apply the default value using the helper function
                    create_or_modify_registry_value(
                        modification.path,
                        modification.key,
                        &modification.value,
                    )
                    .with_context(|| {
                        format!(
                            "{:?} -> Failed to set default value '{}' in '{}'",
                            self.id, modification.key, modification.path
                        )
                    })?;
                    debug!(
                        "{:?} -> Set default value '{}' to {:?} in '{}'.",
                        self.id, modification.key, modification.value, modification.path
                    );
                }
            }
        }

        debug!(
            "{:?} -> Successfully reverted registry tweak to Default.",
            self.id
        );
        Ok(())
    }
}
