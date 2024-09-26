// src/tweaks/registry_tweaks.rs

use std::sync::{Arc, Mutex};

use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey,
};

use super::TweakMethod;
use crate::{
    actions::Tweak,
    errors::RegistryError,
    tweaks::{add_tweak, TweakId},
};

/// Represents a registry tweak, including the registry key, value name, desired value, and default value.
#[derive(Clone, Debug)]
pub struct RegistryTweak {
    /// Full path of the registry key (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
    pub key: String,
    /// Name of the registry value to modify.
    pub name: String,
    /// The value to set when applying the tweak.
    pub tweak_value: RegistryKeyValue,
    /// The default value to revert to when undoing the tweak.
    pub default_value: RegistryKeyValue,
}

/// Enumeration of supported registry key value types.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RegistryKeyValue {
    String(String),
    Dword(u32),
    // Add other types as needed (e.g., Qword, Binary, etc.)
}

impl RegistryTweak {
    /// Checks if the tweak is currently enabled by comparing the current value to the default value.
    /// If the current value matches the default value, the tweak is considered enabled.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation succeeds and the tweak is enabled.
    /// - `Ok(false)` if the operation succeeds and the tweak is disabled.
    pub fn is_registry_tweak_enabled(&self) -> Result<bool, RegistryError> {
        match self.read_current_value() {
            Ok(current_value) => Ok(current_value == self.default_value),
            Err(e) => {
                tracing::error!(
                    "Failed to read current value for tweak '{}': {}",
                    self.name,
                    e
                );
                Ok(false)
            }
        }
    }
    /// Reads the current value of the specified registry key.
    ///
    /// # Returns
    ///
    /// - `Ok(RegistryKeyValue)` with the current value.
    /// - `Err(RegistryTweakError)` if the operation fails.
    pub fn read_current_value(&self) -> Result<RegistryKeyValue, RegistryError> {
        // Extract the hive from the key path (e.g., "HKEY_LOCAL_MACHINE")
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.key.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryError::UnsupportedHive(other.to_string())),
        };

        // Extract the subkey path (everything after the hive)
        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.key.clone()))?;

        // Attempt to open the subkey with read permissions
        let subkey = hkey
            .open_subkey_with_flags(subkey_path, KEY_READ)
            .map_err(|e| {
                RegistryError::KeyOpenError(format!(
                    "Failed to open registry key '{}': {}",
                    self.key, e
                ))
            })?;

        // Depending on the expected type, read the value
        match &self.tweak_value {
            RegistryKeyValue::String(_) => {
                let val: String = subkey.get_value(&self.name).map_err(|e| {
                    RegistryError::ReadValueError(format!(
                        "Failed to read string value '{:.?}': {:.?}",
                        self.tweak_value, e
                    ))
                })?;
                Ok(RegistryKeyValue::String(val))
            }
            RegistryKeyValue::Dword(_) => {
                let val: u32 = subkey.get_value(&self.name).map_err(|e| {
                    RegistryError::ReadValueError(format!(
                        "Failed to read DWORD value '{:.?}': {:.?}",
                        self.tweak_value, e
                    ))
                })?;
                Ok(RegistryKeyValue::Dword(val))
            }
        }
    }

    /// Applies the registry tweak by setting the specified registry value.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(RegistryTweakError)` if the operation fails.
    pub fn apply_registry_tweak(&self) -> Result<(), RegistryError> {
        // Extract the hive from the key path
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.key.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryError::UnsupportedHive(other.to_string())),
        };

        // Extract the subkey path
        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.key.clone()))?;

        // Attempt to open the subkey with read and write permissions
        // If it doesn't exist, attempt to create it
        let subkey = match hkey.open_subkey_with_flags(subkey_path, KEY_READ | KEY_WRITE) {
            Ok(key) => key, // Subkey exists and is opened successfully
            Err(_) => {
                // Subkey does not exist; attempt to create it
                match hkey.create_subkey(subkey_path) {
                    Ok((key, disposition)) => {
                        // Log whether the key was created or already existed
                        match disposition {
                            winreg::enums::RegDisposition::REG_CREATED_NEW_KEY => {
                                tracing::debug!("Created new registry key: {}", self.key);
                            }
                            winreg::enums::RegDisposition::REG_OPENED_EXISTING_KEY => {
                                tracing::debug!("Opened existing registry key: {}", self.key);
                            }
                        }
                        key
                    }
                    Err(e) => {
                        return Err(RegistryError::CreateError(format!(
                            "Failed to create registry key '{:?}': {:?}",
                            self.key, e
                        )))
                    }
                }
            }
        };

        // Now, set the registry value based on its type
        match &self.tweak_value {
            RegistryKeyValue::String(val) => {
                subkey.set_value(&self.name, val).map_err(|e| {
                    RegistryError::SetValueError(format!(
                        "Failed to set string value '{:?}' in key '{:?}': {:?}",
                        self.name, self.key, e
                    ))
                })?;
                tracing::debug!(
                    "Set string value '{:.?}' to '{:.?}' in key '{:.?}'",
                    self.tweak_value,
                    val,
                    self.key
                );
            }
            RegistryKeyValue::Dword(val) => {
                subkey.set_value(&self.name, val).map_err(|e| {
                    RegistryError::SetValueError(format!(
                        "Failed to set DWORD value '{:?}' in key '{:?}': {:?}",
                        self.name, self.key, e
                    ))
                })?;
                tracing::debug!(
                    "Set DWORD value '{:.?}' to '{:.?}' in key '{:.?}'",
                    self.tweak_value,
                    val,
                    self.key
                );
            } // Handle other types as needed
        }

        Ok(())
    }

    /// Reverts the registry tweak by restoring the default registry value.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(RegistryTweakError)` if the operation fails.
    pub fn revert_registry_tweak(&self) -> Result<(), RegistryError> {
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.key.clone()))?;
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryError::UnsupportedHive(other.to_string())),
        };

        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.key.clone()))?;

        // Open the subkey with write permissions to modify the value
        let subkey = hkey
            .open_subkey_with_flags(subkey_path, KEY_WRITE)
            .map_err(|e| {
                RegistryError::KeyOpenError(format!(
                    "Failed to open registry key '{}': {}",
                    self.key, e
                ))
            })?;

        // Set the registry value back to its default
        match &self.default_value {
            RegistryKeyValue::String(val) => subkey
                .set_value(&self.name, val)
                .map_err(|e| RegistryError::SetValueError(e.to_string())),
            RegistryKeyValue::Dword(val) => subkey
                .set_value(&self.name, val)
                .map_err(|e| RegistryError::SetValueError(e.to_string())),
            // Handle other types as needed
        }
    }
}

pub fn initialize_registry_tweaks() -> Vec<Arc<Mutex<Tweak>>> {
    vec![
       add_tweak(
        TweakId::LargeSystemCache,
        "LargeSystemCache".to_string(),
        "Optimizes system memory management by adjusting the LargeSystemCache setting.".to_string(),
        TweakMethod::Registry(RegistryTweak {
            key: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
            name: "LargeSystemCache".to_string(),
            // Windows will act as a server, optimizing for file sharing and network operations, potentially improving RAM disk performance.
            tweak_value: RegistryKeyValue::Dword(1),
            // Windows will favor foreground applications in terms of memory allocation.
            default_value: RegistryKeyValue::Dword(0),
        }),
        false // requires_restart
       ),
       add_tweak(
        TweakId::SystemResponsiveness,
          "SystemResponsiveness".to_string(),
          "Optimizes system responsiveness by adjusting the SystemResponsiveness setting.".to_string(),
          TweakMethod::Registry(RegistryTweak {
                key: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile".to_string(),
                name: "SystemResponsiveness".to_string(),
                // Windows will favor foreground applications in terms of resource allocation.
                tweak_value: RegistryKeyValue::Dword(0),
                // Windows will favor background services in terms of resource allocation.
                default_value: RegistryKeyValue::Dword(20),
          }),
          false // requires_restart
         ),
         add_tweak(
            TweakId::DisableHWAcceleration,
            "DisableHWAcceleration".to_string(),
            "Disables hardware acceleration for the current user.".to_string(),
            TweakMethod::Registry(RegistryTweak {
                    key: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics".to_string(),
                    name: "DisableHWAcceleration".to_string(),
                    // Hardware acceleration is disabled.
                    tweak_value: RegistryKeyValue::Dword(1),
                    // Hardware acceleration is enabled.
                    default_value: RegistryKeyValue::Dword(0),
            }),
            false // requires_restart
            ),
            add_tweak(
            TweakId::Win32PrioritySeparation,
            "Win32PrioritySeparation".to_string(),
            "Optimizes system responsiveness by adjusting the Win32PrioritySeparation setting.".to_string(),
            TweakMethod::Registry(RegistryTweak {
                    key: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\PriorityControl".to_string(),
                    name: "Win32PrioritySeparation".to_string(),
                    // Foreground applications will receive priority over background services.
                    tweak_value: RegistryKeyValue::Dword(26),
                    // Background services will receive priority over foreground applications.
                    default_value: RegistryKeyValue::Dword(2),
            }),
            false // requires_restart
            ),
            add_tweak(
                TweakId::DisableLowDiskCheck,
                "DisableLowDiskCheck".to_string(),
                "Disables the low disk space check for the current user.".to_string(),
                TweakMethod::Registry(RegistryTweak {
                key: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer".to_string(),
                name: "NoLowDiskSpaceChecks".to_string(),
                // Low disk space check is disabled.
                tweak_value: RegistryKeyValue::Dword(1),
                // Low disk space check is enabled.
                default_value: RegistryKeyValue::Dword(0),
                }),
                false // requires_restart
            ),
            add_tweak(
                TweakId::DisableCoreParking,
                "DisableCoreParking".to_string(),
                "Disables core parking to improve system performance.".to_string(),
                TweakMethod::Registry(RegistryTweak {
                key: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583".to_string(),
                name: "ValueMax".to_string(),
                // Core parking is disabled.
                tweak_value: RegistryKeyValue::Dword(0),
                // Core parking is enabled.
                default_value: RegistryKeyValue::Dword(64),
                }),
                true // requires_restart
            ),
            ]
}
