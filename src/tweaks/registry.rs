// src/tweaks/registry.rs

use anyhow::{Context, Result};
use tracing::{debug, error, info, trace};
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey, RegValue,
};

use super::definitions::TweakId;
use crate::tweaks::TweakMethod;

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

/// Enumeration of supported registry key value types.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RegistryKeyValue {
    Dword(u32),
    Binary(Vec<u8>),
}

impl RegistryTweak<'_> {
    /// Parses the full registry path into hive and subkey path.
    ///
    /// # Parameters
    ///
    /// - `path`: The full registry path (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
    ///
    /// # Returns
    ///
    /// - `Ok((&str, &str))` containing the hive and subkey path.
    /// - `Err(anyhow::Error)` if parsing fails.
    fn parse_registry_path(path: &str) -> Result<(&str, &str)> {
        let components: Vec<&str> = path.split('\\').collect();
        if components.len() < 2 {
            anyhow::bail!("Invalid registry path: {}", path);
        }
        let hive = components[0];
        let subkey_path = &path[hive.len() + 1..]; // +1 to skip the backslash
        Ok((hive, subkey_path))
    }

    /// Maps the hive string to the corresponding `RegKey`.
    ///
    /// # Parameters
    ///
    /// - `hive`: The registry hive string (e.g., "HKEY_LOCAL_MACHINE").
    ///
    /// # Returns
    ///
    /// - `Ok(RegKey)` corresponding to the hive.
    /// - `Err(anyhow::Error)` if the hive is unsupported.
    fn get_hkey(hive: &str) -> Result<RegKey> {
        match hive {
            "HKEY_LOCAL_MACHINE" => Ok(RegKey::predef(HKEY_LOCAL_MACHINE)),
            "HKEY_CURRENT_USER" => Ok(RegKey::predef(HKEY_CURRENT_USER)),
            other => {
                error!("Unsupported registry hive '{}'.", other);
                anyhow::bail!("Unsupported registry hive '{}'.", other)
            }
        }
    }

    /// Opens a subkey with specified access.
    ///
    /// # Parameters
    ///
    /// - `hive`: The registry hive.
    /// - `subkey_path`: The subkey path within the hive.
    /// - `access`: Access flags (e.g., `KEY_READ`, `KEY_WRITE`).
    ///
    /// # Returns
    ///
    /// - `Ok(RegKey)` if successful.
    /// - `Err(anyhow::Error)` if opening fails.
    fn open_subkey(&self, hive: &str, subkey_path: &str, access: u32) -> Result<RegKey> {
        let hkey = Self::get_hkey(hive)?;
        hkey.open_subkey_with_flags(subkey_path, access)
            .with_context(|| {
                format!(
                    "Failed to open subkey '{}' with access {}",
                    subkey_path, access
                )
            })
    }

    /// Creates a subkey if it doesn't exist.
    ///
    /// # Parameters
    ///
    /// - `hive`: The registry hive.
    /// - `subkey_path`: The subkey path within the hive.
    ///
    /// # Returns
    ///
    /// - `Ok(RegKey)` corresponding to the created or opened subkey.
    /// - `Err(anyhow::Error)` if creation fails.
    fn create_subkey(&self, hive: &str, subkey_path: &str) -> Result<RegKey> {
        let hkey = Self::get_hkey(hive)?;
        let (key, disposition) = hkey
            .create_subkey(subkey_path)
            .with_context(|| format!("Failed to create or open subkey '{}'", subkey_path))?;
        match disposition {
            winreg::enums::RegDisposition::REG_CREATED_NEW_KEY => {
                info!(
                    "{:?} -> Created new registry key '{}'.",
                    self.id, subkey_path
                );
            }
            winreg::enums::RegDisposition::REG_OPENED_EXISTING_KEY => {
                debug!(
                    "{:?} -> Opened existing registry key '{}'.",
                    self.id, subkey_path
                );
            }
        }
        Ok(key)
    }

    /// Sets a registry value.
    ///
    /// # Parameters
    ///
    /// - `key`: The `RegKey` to modify.
    /// - `value_name`: The name of the registry value.
    /// - `value`: The `RegistryKeyValue` to set.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if successful.
    /// - `Err(anyhow::Error)` if setting fails.
    fn set_value(&self, key: &RegKey, value_name: &str, value: &RegistryKeyValue) -> Result<()> {
        match value {
            RegistryKeyValue::Dword(v) => key
                .set_value(value_name, v)
                .with_context(|| format!("Failed to set DWORD value '{}' to '{}'", value_name, v)),
            RegistryKeyValue::Binary(data) => {
                // Set a Binary value
                key.set_raw_value(
                    value_name,
                    &RegValue {
                        bytes: data.clone(),
                        vtype: winreg::enums::REG_BINARY, // Corrected field name
                    },
                )
                .with_context(|| {
                    format!(
                        "Failed to set Binary value '{}' to '{:?}'",
                        value_name, data
                    )
                })
            }
        }
    }

    /// Gets a registry value.
    ///
    /// # Parameters
    ///
    /// - `key`: The `RegKey` to read from.
    /// - `value_name`: The name of the registry value.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(RegistryKeyValue))` if the value exists.
    /// - `Ok(None)` if the value does not exist.
    /// - `Err(anyhow::Error)` if reading fails.
    fn get_value(&self, key: &RegKey, value_name: &str) -> Result<Option<RegistryKeyValue>> {
        // Attempt to read as DWORD
        match key.get_value::<u32, &str>(value_name) {
            Ok(val) => Ok(Some(RegistryKeyValue::Dword(val))),
            Err(ref e) if Self::is_not_found_error(e) => {
                // If DWORD read fails due to NotFound, attempt to read as Binary
                match key.get_raw_value(value_name) {
                    Ok(reg_val) => {
                        if reg_val.vtype == winreg::enums::REG_BINARY {
                            Ok(Some(RegistryKeyValue::Binary(reg_val.bytes)))
                        } else {
                            // Unsupported type
                            Err(anyhow::anyhow!(
                                "Unsupported registry value type for '{}'",
                                value_name
                            ))
                        }
                    }
                    Err(ref e_inner) if Self::is_not_found_error(e_inner) => Ok(None),
                    Err(e_inner) => Err(anyhow::Error::from(e_inner)).with_context(|| {
                        format!("Failed to get registry value '{}' as Binary", value_name)
                    }),
                }
            }
            Err(e) => Err(anyhow::Error::from(e))
                .with_context(|| format!("Failed to get registry value '{}' as DWORD", value_name)),
        }
    }

    /// Helper method to determine if an error is a NotFound error.
    fn is_not_found_error(e: &(dyn std::error::Error + 'static)) -> bool {
        if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
            io_error.kind() == std::io::ErrorKind::NotFound
        } else {
            false
        }
    }

    /// Deletes a registry value.
    ///
    /// # Parameters
    ///
    /// - `key`: The `RegKey` to modify.
    /// - `value_name`: The name of the registry value to delete.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if successful.
    /// - `Err(anyhow::Error)` if deletion fails.
    fn delete_value(&self, key: &RegKey, value_name: &str) -> Result<()> {
        key.delete_value(value_name)
            .with_context(|| format!("Failed to delete value '{}'", value_name))
    }

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
                let (hive, subkey_path) = Self::parse_registry_path(modification.path)
                    .with_context(|| {
                        format!("Failed to parse registry path '{}'", modification.path)
                    })?;
                let subkey = self
                    .open_subkey(hive, subkey_path, KEY_READ)
                    .with_context(|| format!("Failed to open subkey '{}'", modification.path))?;
                let value = self
                    .get_value(&subkey, modification.key)
                    .with_context(|| {
                        format!(
                            "Failed to read value '{}' from '{}'",
                            modification.key, modification.path
                        )
                    })?
                    .unwrap_or_else(|| {
                        // Provide a sensible default based on the target_value's variant
                        match modification.target_value {
                            RegistryKeyValue::Dword(_) => RegistryKeyValue::Dword(0),
                            RegistryKeyValue::Binary(_) => RegistryKeyValue::Binary(Vec::new()),
                        }
                    });
                Ok(value)
            })
            .collect()
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
        info!(
            "{:?} -> Initiating rollback for {} operation.",
            self.id, operation
        );
        for (modification, original_value) in modifications.iter().rev() {
            let (hive, subkey_path) =
                Self::parse_registry_path(modification.path).with_context(|| {
                    format!("Failed to parse registry path '{}'", modification.path)
                })?;
            let subkey = match self.open_subkey(hive, subkey_path, KEY_WRITE) {
                Ok(k) => k,
                Err(_) => {
                    error!("{:?} -> Registry key '{}' does not exist during rollback. Cannot revert modification '{}'.",
                        self.id, modification.path, modification.key);
                    anyhow::bail!(
                        "Registry key '{}' does not exist during rollback.",
                        modification.path
                    );
                }
            };

            match original_value {
                Some(val) => {
                    self.set_value(&subkey, modification.key, val)
                        .with_context(|| {
                            format!(
                                "Failed to restore value '{}' in '{}'",
                                modification.key, modification.path
                            )
                        })?;
                    info!(
                        "{:?} -> Restored value '{}' to {:?} in '{}'.",
                        self.id, modification.key, val, modification.path
                    );
                }
                None => match self.delete_value(&subkey, modification.key) {
                    Ok(_) => {
                        info!(
                            "{:?} -> Deleted value '{}' in '{}'.",
                            self.id, modification.key, modification.path
                        );
                    }
                    Err(e) => {
                        if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
                            if io_error.kind() == std::io::ErrorKind::NotFound {
                                info!("{:?} -> Value '{}' already does not exist in '{}'. No action needed.", self.id, modification.key, modification.path);
                            } else {
                                let error_msg = format!(
                                    "Failed to delete value '{}' in '{}': {}",
                                    modification.key, modification.path, e
                                );
                                tracing::error!("{:?} -> {}", self.id, error_msg);
                                return Err(anyhow::Error::msg(error_msg));
                            }
                        } else {
                            let error_msg = format!(
                                "Failed to delete value '{}' in '{}': {}",
                                modification.key, modification.path, e
                            );
                            tracing::error!("{:?} -> {}", self.id, error_msg);
                            return Err(anyhow::Error::msg(error_msg));
                        }
                    }
                },
            }
        }
        info!(
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
        info!("{:?} -> Determining if registry tweak is enabled.", self.id);

        for modification in &self.modifications {
            let (hive, subkey_path) =
                Self::parse_registry_path(modification.path).with_context(|| {
                    format!("Failed to parse registry path '{}'", modification.path)
                })?;

            // Attempt to open the subkey with read access
            let subkey = match self.open_subkey(hive, subkey_path, KEY_READ) {
                Ok(k) => k,
                Err(e) => {
                    if modification.default_value.is_none() {
                        // If default_value is None and the key doesn't exist, the tweak is disabled
                        info!(
                            "{:?} -> Registry key '{}' does not exist. Modification '{}' is disabled.",
                            self.id, modification.path, modification.key
                        );
                        return Ok(false);
                    } else {
                        // For modifications with a default value, failing to open the key is an error
                        tracing::error!(
                            "{:?} -> Failed to open subkey '{}': {}",
                            self.id,
                            modification.path,
                            e
                        );
                        return Err(e);
                    }
                }
            };

            if modification.default_value.is_some() {
                // For modifications with a default value, compare the current value with the target value
                match self.get_value(&subkey, modification.key)? {
                    Some(current_val) if current_val == modification.target_value => {
                        info!(
                            "{:?} -> Modification '{}' is enabled. Value matches {:?}.",
                            self.id, modification.key, modification.target_value
                        );
                    }
                    Some(current_val) => {
                        info!(
                            "{:?} -> Modification '{}' is disabled. Expected {:?}, found {:?}.",
                            self.id, modification.key, modification.target_value, current_val
                        );
                        return Ok(false);
                    }
                    None => {
                        info!(
                            "{:?} -> Modification '{}' is disabled. Value does not exist.",
                            self.id, modification.key
                        );
                        return Ok(false);
                    }
                }
            } else {
                // For modifications without a default value, check if the registry value exists
                let exists = self.get_value(&subkey, modification.key)?.is_some();
                if exists {
                    info!(
                        "{:?} -> Modification '{}' is enabled. Value exists.",
                        self.id, modification.key
                    );
                } else {
                    info!(
                        "{:?} -> Modification '{}' is disabled. Value does not exist.",
                        self.id, modification.key
                    );
                    return Ok(false);
                }
            }
        }

        info!("{:?} -> All modifications are enabled.", self.id);
        Ok(true) // All modifications are enabled
    }

    /// Applies the registry tweak by setting the specified registry values atomically.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations succeed.
    /// - `Err(anyhow::Error)` if any operation fails, after attempting rollback.
    fn apply(&self) -> Result<(), anyhow::Error> {
        info!("Applying registry tweak '{:?}'.", self.id);
        let mut applied_modifications = Vec::new();

        // Wrap the apply logic in a closure to handle errors and perform rollback
        let result: Result<(), anyhow::Error> = (|| -> Result<(), anyhow::Error> {
            for modification in &self.modifications {
                // Extract the hive and subkey path from the registry path
                let (hive, subkey_path) = Self::parse_registry_path(modification.path)
                    .with_context(|| {
                        format!("Failed to parse registry path '{}'", modification.path)
                    })?;

                // Open subkey for reading original values
                let subkey_read = match self.open_subkey(hive, subkey_path, KEY_READ) {
                    Ok(k) => k,
                    Err(e) => {
                        let error_msg = format!(
                            "Failed to open registry key '{}' for reading: {}",
                            modification.path, e
                        );
                        tracing::error!("{:?} -> {}", self.id, error_msg);
                        return Err(anyhow::Error::msg(error_msg));
                    }
                };

                // Read and store the original value
                let original_value = self
                    .get_value(&subkey_read, modification.key)
                    .with_context(|| {
                        format!(
                            "Failed to read original value '{}' from '{}'",
                            modification.key, modification.path
                        )
                    })?;

                // Open or create subkey for writing
                let subkey_write = match self.open_subkey(hive, subkey_path, KEY_WRITE) {
                    Ok(k) => k,
                    Err(_) => {
                        // Subkey does not exist; attempt to create it
                        match self.create_subkey(hive, subkey_path) {
                            Ok(k) => k,
                            Err(e) => {
                                let error_msg = format!(
                                    "Failed to create registry key '{}': {}",
                                    modification.path, e
                                );
                                tracing::error!("{:?} -> {}", self.id, error_msg);
                                return Err(anyhow::Error::msg(error_msg));
                            }
                        }
                    }
                };

                // Apply the tweak value
                self.set_value(&subkey_write, modification.key, &modification.target_value)
                    .with_context(|| {
                        format!(
                            "Failed to set value '{}' in '{}'",
                            modification.key, modification.path
                        )
                    })?;

                info!(
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
            tracing::error!(
                "{:?} -> Error occurred during apply: {}. Attempting rollback.",
                self.id,
                e
            );

            // Attempt to rollback
            if let Err(rollback_err) = self.rollback(&applied_modifications, "apply") {
                // Rollback failed
                tracing::error!(
                    "{:?} -> Failed to rollback after apply error: {}",
                    self.id,
                    rollback_err
                );
                // Return an error indicating both the original error and the rollback error
                anyhow::bail!("Apply failed: {}. Rollback failed: {}", e, rollback_err);
            } else {
                tracing::info!(
                    "{:?} -> Successfully rolled back after apply error.",
                    self.id
                );
            }
            // Return the original error
            return Err(e);
        }

        tracing::info!("Successfully applied registry tweak '{:?}'.", self.id);
        Ok(())
    }

    /// Reverts the registry tweak by restoring the default registry values or deleting them if no defaults are provided, atomically.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations succeed.
    /// - `Err(anyhow::Error)` if any operation fails, after attempting rollback.
    fn revert(&self) -> Result<(), anyhow::Error> {
        info!("{:?} -> Reverting registry tweak.", self.id);
        let mut reverted_modifications = Vec::new();

        // Wrap the revert logic in a closure to handle errors and perform rollback
        let result: Result<(), anyhow::Error> = (|| -> Result<(), anyhow::Error> {
            for modification in &self.modifications {
                // Extract the hive and subkey path from the registry path
                let (hive, subkey_path) = Self::parse_registry_path(modification.path)
                    .with_context(|| {
                        format!("Failed to parse registry path '{}'", modification.path)
                    })?;

                // Open subkey for reading current values before reverting
                let subkey_read = match self.open_subkey(hive, subkey_path, KEY_READ) {
                    Ok(k) => k,
                    Err(e) => {
                        let error_msg = format!(
                            "Failed to open registry key '{}' for reading during revert: {}",
                            modification.path, e
                        );
                        tracing::error!("{:?} -> {}", self.id, error_msg);
                        return Err(anyhow::Error::msg(error_msg));
                    }
                };

                // Read and store the current value before reverting
                let current_value = self
                    .get_value(&subkey_read, modification.key)
                    .with_context(|| {
                        format!(
                            "Failed to read current value '{}' from '{}'",
                            modification.key, modification.path
                        )
                    })?;

                // Open or create subkey for writing
                let subkey_write = match self.open_subkey(hive, subkey_path, KEY_WRITE) {
                    Ok(k) => k,
                    Err(_) => {
                        // Subkey does not exist; attempt to create it
                        match self.create_subkey(hive, subkey_path) {
                            Ok(k) => k,
                            Err(e) => {
                                let error_msg = format!(
                                    "Failed to create registry key '{}': {}",
                                    modification.path, e
                                );
                                tracing::error!("{:?} -> {}", self.id, error_msg);
                                return Err(anyhow::Error::msg(error_msg));
                            }
                        }
                    }
                };

                // Revert the modification
                match &modification.default_value {
                    Some(default_val) => {
                        // Restore the default value
                        self.set_value(&subkey_write, modification.key, default_val)
                            .with_context(|| {
                                format!(
                                    "Failed to restore default value '{}' in '{}'",
                                    modification.key, modification.path
                                )
                            })?;
                        info!(
                            "{:?} -> Restored value '{}' to {:?} in '{}'.",
                            self.id, modification.key, default_val, modification.path
                        );
                    }
                    None => match self.delete_value(&subkey_write, modification.key) {
                        Ok(_) => {
                            info!(
                                "{:?} -> Deleted value '{}' in '{}'.",
                                self.id, modification.key, modification.path
                            );
                        }
                        Err(e) => {
                            if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
                                if io_error.kind() == std::io::ErrorKind::NotFound {
                                    info!("{:?} -> Value '{}' already does not exist in '{}'. No action needed.", self.id, modification.key, modification.path);
                                } else {
                                    let error_msg = format!(
                                        "Failed to delete value '{}' in '{}': {}",
                                        modification.key, modification.path, e
                                    );
                                    tracing::error!("{:?} -> {}", self.id, error_msg);
                                    return Err(anyhow::Error::msg(error_msg));
                                }
                            } else {
                                let error_msg = format!(
                                    "Failed to delete value '{}' in '{}': {}",
                                    modification.key, modification.path, e
                                );
                                tracing::error!("{:?} -> {}", self.id, error_msg);
                                return Err(anyhow::Error::msg(error_msg));
                            }
                        }
                    },
                }

                // Record the successfully reverted modification along with its current value
                reverted_modifications.push((modification.clone(), current_value));
            }
            Ok(())
        })();

        if let Err(e) = result {
            // An error occurred during revert
            tracing::error!(
                "{:?} -> Error occurred during revert: {}. Attempting rollback.",
                self.id,
                e
            );

            // Attempt to rollback
            if let Err(rollback_err) = self.rollback(&reverted_modifications, "revert") {
                // Rollback failed
                tracing::error!(
                    "{:?} -> Failed to rollback after revert error: {}",
                    self.id,
                    rollback_err
                );
                // Return an error indicating both the original error and the rollback error
                anyhow::bail!("Revert failed: {}. Rollback failed: {}", e, rollback_err);
            } else {
                tracing::info!(
                    "{:?} -> Successfully rolled back after revert error.",
                    self.id
                );
            }
            // Return the original error
            return Err(e);
        }

        tracing::info!("{:?} -> Successfully reverted registry tweak.", self.id);
        Ok(())
    }
}
