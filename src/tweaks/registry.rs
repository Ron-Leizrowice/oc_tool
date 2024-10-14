// src/tweaks/registry.rs

use anyhow::{Context, Result};
use tracing::{debug, error, info, trace};
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey,
};

use crate::tweaks::{TweakId, TweakMethod};

/// Defines a set of modifications to the Windows registry, which in combination
/// make up a single tweak.
#[derive(Debug)]
pub struct RegistryTweak {
    /// Unique ID for the tweak
    pub id: TweakId,
    pub(crate) modifications: Vec<RegistryModification>,
}

/// Represents a single registry modification, including the registry key, value name, desired value, and default value.
/// If `default_value` is `None`, the modification is considered enabled if the registry value exists.
/// Reverting such a tweak involves deleting the registry value.
#[derive(Debug, Clone)]
pub struct RegistryModification {
    /// Full path of the registry key (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
    pub path: String,
    /// Name of the registry value to modify.
    pub key: String,
    /// The value to set when applying the tweak.
    pub target_value: RegistryKeyValue,
    /// The default value to revert to when undoing the tweak.
    /// If `None`, reverting deletes the registry value.
    pub default_value: Option<RegistryKeyValue>,
}

/// Enumeration of supported registry key value types.
/// Currently, only `Dword` is implemented.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RegistryKeyValue {
    Dword(u32),
}

impl RegistryTweak {
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
                info!("{:?} -> Created new registry key '{}'.", self.id, hive);
            }
            winreg::enums::RegDisposition::REG_OPENED_EXISTING_KEY => {
                debug!("{:?} -> Opened existing registry key '{}'.", self.id, hive);
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
                .with_context(|| format!("Failed to set DWORD value '{}' to '{}' ", value_name, v)),
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
        match key.get_value::<u32, &str>(value_name) {
            Ok(val) => Ok(Some(RegistryKeyValue::Dword(val))),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(anyhow::Error::from(e))
                .with_context(|| format!("Failed to get value '{}'", value_name)),
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
                let (hive, subkey_path) = Self::parse_registry_path(&modification.path)
                    .with_context(|| {
                        format!("Failed to parse registry path '{}'", modification.path)
                    })?;
                let subkey = self
                    .open_subkey(hive, subkey_path, KEY_READ)
                    .with_context(|| format!("Failed to open subkey '{}'", modification.path))?;
                let value = self
                    .get_value(&subkey, &modification.key)
                    .with_context(|| {
                        format!(
                            "Failed to read value '{}' from '{}'",
                            modification.key, modification.path
                        )
                    })?
                    .unwrap_or(RegistryKeyValue::Dword(0)); // Default to 0 if not found
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
                Self::parse_registry_path(&modification.path).with_context(|| {
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
                    self.set_value(&subkey, &modification.key, val)
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
                None => match self.delete_value(&subkey, &modification.key) {
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

impl TweakMethod for RegistryTweak {
    /// Checks if the tweak is currently enabled.
    ///
    /// - If all of the initial values of the registry keys match the tweak's `target_values`, the tweak is enabled.
    /// - If any of the initial values do not match the `target_values`, the tweak is disabled.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the tweak is enabled.
    /// - `Ok(false)` if the tweak is disabled.
    /// - `Err(anyhow::Error)` if an error occurs while reading the registry.
    fn initial_state(&self) -> Result<bool, anyhow::Error> {
        info!("{:?} -> Determining if registry tweak is enabled.", self.id);
        let current_values = match self.read_current_values() {
            Ok(vals) => vals,
            Err(e) => {
                // Log the error and assume tweak is disabled
                tracing::error!(
                    "{:?} -> Failed to read current registry values: {}. Assuming tweak is disabled.",
                    self.id,
                    e
                );
                return Ok(false);
            }
        };

        for (i, modification) in self.modifications.iter().enumerate() {
            let current_value = &current_values[i];
            let target_value = &modification.target_value;

            if current_value != target_value {
                tracing::info!(
                    "{:?} -> Modification '{}' is disabled. Expected {:?}, found {:?}.",
                    self.id,
                    modification.key,
                    target_value,
                    current_value
                );
                return Ok(false); // If any value does not match, tweak is disabled
            } else {
                tracing::info!(
                    "{:?} -> Modification '{}' is enabled. Value matches {:?}.",
                    self.id,
                    modification.key,
                    target_value
                );
            }
        }

        tracing::info!(
            "{:?} -> All modifications match their target values. Tweak is enabled.",
            self.id
        );
        Ok(true) // All values match, tweak is enabled
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
                let (hive, subkey_path) = Self::parse_registry_path(&modification.path)
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
                    .get_value(&subkey_read, &modification.key)
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
                self.set_value(&subkey_write, &modification.key, &modification.target_value)
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
                let (hive, subkey_path) = Self::parse_registry_path(&modification.path)
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
                    .get_value(&subkey_read, &modification.key)
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

                // Revert the Dword modification based on whether a default value exists
                match &modification.default_value {
                    Some(default_val) => match default_val {
                        RegistryKeyValue::Dword(v) => {
                            self.set_value(&subkey_write, &modification.key, default_val)
                                .with_context(|| format!("Failed to restore default DWORD value '{}' in key '{}': {}", modification.key, modification.path, v))?;
                            tracing::info!(
                                "{:?} -> Restored value '{}' to {:?} in '{}'.",
                                self.id,
                                modification.key,
                                v,
                                modification.path
                            );
                        }
                    },
                    None => {
                        // Delete the value as it did not exist by default
                        match self.delete_value(&subkey_write, &modification.key) {
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
                        }
                    }
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
