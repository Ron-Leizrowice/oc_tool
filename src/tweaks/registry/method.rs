// src/tweaks/registry.rs

use anyhow::{Context, Result};
use tracing::{debug, error, trace};

use crate::{
    tweaks::{TweakId, TweakMethod},
    utils::registry::{
        create_or_modify_registry_value, delete_registry_value, read_registry_value,
        RegistryKeyValue,
    },
};

/// Defines a set of modifications to the Windows registry, which in combination
/// make up a single tweak.
#[derive(Debug)]
pub struct RegistryTweak<'a> {
    /// Unique ID for the tweak
    pub id: TweakId,
    pub(crate) modifications: Vec<RegistryModification<'a>>,
}

/// Represents a single registry modification, including the registry key, value name, desired value, and default value.
/// If `default_value` is `None`, the modification is considered enabled if the registry value exists.
/// Reverting such a tweak involves deleting the registry value.
#[derive(Debug, Clone)]
pub struct RegistryModification<'a> {
    /// Full path of the registry key (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
    pub path: &'a str,
    /// Name of the registry value to modify.
    pub key: &'a str,
    /// The value to set when applying the tweak.
    pub target_value: RegistryKeyValue,
    /// The default value to revert to when undoing the tweak.
    /// If `None`, reverting deletes the registry value.
    pub default_value: Option<RegistryKeyValue>,
}

impl RegistryTweak<'_> {
    /// Reads the current values of all registry modifications in the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<RegistryKeyValue>)` with the current values.
    /// - `Err(anyhow::Error)` if any operation fails.
    pub fn read_current_values(&self) -> Result<Vec<RegistryKeyValue>> {
        trace!("{:?} -> Reading current values of registry tweak.", self.id);
        self.modifications
            .iter()
            .map(|modification| {
                // Get the value using the helper function
                let value =
                    read_registry_value(modification.path, modification.key).context(format!(
                        "Failed to read value '{}' from '{}'",
                        modification.key, modification.path
                    ))?;
                value.ok_or_else(|| {
                    let err_msg = format!(
                        "Value '{}' not found in '{}'",
                        modification.key, modification.path
                    );
                    tracing::error!("{:?} -> {}", self.id, err_msg);
                    anyhow::anyhow!(err_msg)
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    /// Rolls back previously applied modifications.
    ///
    /// # Parameters
    ///
    /// - `modifications`: A slice of tuples containing the `RegistryModification` and their original values.
    /// - `operation`: A string indicating the operation ("apply" or "revert") for logging purposes.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all rollback operations succeed.
    /// - `Err(anyhow::Error)` if any rollback operation fails.
    fn rollback(
        &self,
        modifications: &[(RegistryModification, Option<RegistryKeyValue>)],
        operation: &str,
    ) -> Result<()> {
        debug!(
            "{:?} -> Initiating rollback for {} operation.",
            self.id, operation
        );
        for (modification, original_value) in modifications.iter().rev() {
            match original_value {
                Some(val) => {
                    create_or_modify_registry_value(modification.path, modification.key, val)
                        .context(format!(
                            "Failed to restore value '{}' in '{}'",
                            modification.key, modification.path
                        ))?;
                    tracing::debug!(
                        "{:?} -> Restored value '{}' to {:?} in '{}'.",
                        self.id,
                        modification.key,
                        val,
                        modification.path
                    );
                }
                None => {
                    delete_registry_value(modification.path, modification.key).context(format!(
                        "Failed to delete value '{}' in '{}'",
                        modification.key, modification.path
                    ))?;
                    tracing::debug!(
                        "{:?} -> Deleted value '{}' in '{}'.",
                        self.id,
                        modification.key,
                        modification.path
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
    /// Checks if the tweak is currently enabled.
    ///
    /// - If `default_value` is `Some`, compare `current_value` with `target_value`.
    /// - If `default_value` is `None`, check if the registry value exists.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the tweak is enabled.
    /// - `Ok(false)` if the tweak is disabled.
    /// - `Err(anyhow::Error)` if an error occurs while reading the registry.
    fn initial_state(&self) -> Result<bool, anyhow::Error> {
        debug!("{:?} -> Determining if registry tweak is enabled.", self.id);

        for modification in &self.modifications {
            if modification.default_value.is_some() {
                // For modifications with a default value, compare the current value with the target value
                match read_registry_value(modification.path, modification.key)? {
                    Some(current_val) if current_val == modification.target_value => {
                        debug!(
                            "{:?} -> Modification '{}' is enabled. Value matches {:?}.",
                            self.id, modification.key, modification.target_value
                        );
                    }
                    Some(current_val) => {
                        debug!(
                            "{:?} -> Modification '{}' is disabled. Expected {:?}, found {:?}.",
                            self.id, modification.key, modification.target_value, current_val
                        );
                        return Ok(false);
                    }
                    None => {
                        debug!(
                            "{:?} -> Modification '{}' is disabled. Value does not exist.",
                            self.id, modification.key
                        );
                        return Ok(false);
                    }
                }
            } else {
                // For modifications without a default value, check if the registry value exists
                let exists = read_registry_value(modification.path, modification.key)?.is_some();
                if exists {
                    debug!(
                        "{:?} -> Modification '{}' is enabled. Value exists.",
                        self.id, modification.key
                    );
                } else {
                    debug!(
                        "{:?} -> Modification '{}' is disabled. Value does not exist.",
                        self.id, modification.key
                    );
                    return Ok(false);
                }
            }
        }

        tracing::debug!("{:?} -> All modifications are enabled.", self.id);
        Ok(true) // All modifications are enabled
    }

    /// Applies the registry tweak by setting the specified registry values atomically.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations succeed.
    /// - `Err(anyhow::Error)` if any operation fails, after attempting rollback.
    fn apply(&self) -> Result<(), anyhow::Error> {
        debug!("Applying registry tweak '{:?}'.", self.id);
        let mut applied_modifications = Vec::new();

        // Wrap the apply logic in a closure to handle errors and perform rollback
        let result: Result<(), anyhow::Error> = (|| -> Result<(), anyhow::Error> {
            for modification in &self.modifications {
                // Read and store the original value
                let original_value = read_registry_value(modification.path, modification.key)
                    .context(format!(
                        "Failed to read original value '{}' from '{}'",
                        modification.key, modification.path
                    ))?;

                // Apply the tweak value using the helper function
                create_or_modify_registry_value(
                    modification.path,
                    modification.key,
                    &modification.target_value,
                )
                .with_context(|| {
                    format!(
                        "Failed to set value '{}' in '{}'",
                        modification.key, modification.path
                    )
                })?;

                debug!(
                    "{:?} -> Set value '{}' to {:?} in '{}'.",
                    self.id, modification.key, modification.target_value, modification.path
                );

                // Record the successfully applied modification along with its original value
                applied_modifications.push((modification.clone(), original_value));
            }
            Ok(())
        })();

        if let Err(e) = result {
            // An error occurred during apply
            error!(
                "{:?} -> Error occurred during apply: {}. Attempting rollback.",
                self.id, e
            );

            // Attempt to rollback
            if let Err(rollback_err) = self.rollback(&applied_modifications, "apply") {
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

        debug!("{:?} -> Successfully applied registry tweak.", self.id);
        Ok(())
    }

    /// Reverts the registry tweak by restoring the default registry values or deleting them if no defaults are provided, atomically.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations succeed.
    /// - `Err(anyhow::Error)` if any operation fails, after attempting rollback.
    fn revert(&self) -> Result<(), anyhow::Error> {
        debug!("{:?} -> Reverting registry tweak.", self.id);
        let mut reverted_modifications = Vec::new();

        // Wrap the revert logic in a closure to handle errors and perform rollback
        let result: Result<(), anyhow::Error> = (|| -> Result<(), anyhow::Error> {
            for modification in &self.modifications {
                // Read and store the current value before reverting
                let current_value = read_registry_value(modification.path, modification.key)
                    .with_context(|| {
                        format!(
                            "Failed to read current value '{}' from '{}'",
                            modification.key, modification.path
                        )
                    })?;

                // Revert the modification using helper functions
                match &modification.default_value {
                    Some(default_val) => {
                        // Restore the default value
                        create_or_modify_registry_value(
                            modification.path,
                            modification.key,
                            default_val,
                        )
                        .with_context(|| {
                            format!(
                                "Failed to restore default value '{}' in '{}'",
                                modification.key, modification.path
                            )
                        })?;
                        debug!(
                            "{:?} -> Restored value '{}' to {:?} in '{}'.",
                            self.id, modification.key, default_val, modification.path
                        );
                    }
                    None => {
                        // Delete the registry value
                        delete_registry_value(modification.path, modification.key).with_context(
                            || {
                                format!(
                                    "Failed to delete value '{}' in '{}'",
                                    modification.key, modification.path
                                )
                            },
                        )?;
                        debug!(
                            "{:?} -> Deleted value '{}' in '{}'.",
                            self.id, modification.key, modification.path
                        );
                    }
                }

                // Record the successfully reverted modification along with its current value
                reverted_modifications.push((modification.clone(), current_value));
            }
            Ok(())
        })();

        if let Err(e) = result {
            // An error occurred during revert
            error!(
                "{:?} -> Error occurred during revert: {}. Attempting rollback.",
                self.id, e
            );

            // Attempt to rollback
            if let Err(rollback_err) = self.rollback(&reverted_modifications, "revert") {
                // Rollback failed
                error!(
                    "{:?} -> Failed to rollback after revert error: {}",
                    self.id, rollback_err
                );
                // Return an error indicating both the original error and the rollback error
                anyhow::bail!("Revert failed: {}. Rollback failed: {}", e, rollback_err);
            } else {
                debug!(
                    "{:?} -> Successfully rolled back after revert error.",
                    self.id
                );
            }
            // Return the original error
            return Err(e);
        }

        debug!("{:?} -> Successfully reverted registry tweak.", self.id);
        Ok(())
    }
}
