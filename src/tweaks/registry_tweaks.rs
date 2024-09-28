// src/tweaks/registry_tweaks.rs

use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey,
};

use crate::{errors::RegistryError, tweaks::TweakId};

/// Represents a registry tweak, including the registry key, value name, desired value, and default value.
#[derive(Clone, Debug)]
pub struct RegistryTweak {
    /// Display name of the tweak.
    pub name: String,
    /// Description of what the tweak does.
    pub description: String,
    /// Full path of the registry key (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
    pub path: String,
    /// Key of the registry value to modify.
    pub key: String,
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
    /// If the value does not exist, assume the tweak is disabled.
    pub fn is_registry_tweak_enabled(&self, id: TweakId) -> Result<bool, RegistryError> {
        tracing::info!("{:?} -> Determining if registry tweak is enabled.", id);
        match self.read_current_value(id) {
            Ok(current_value) => {
                let is_enabled = current_value == self.default_value;
                tracing::info!(
                    "{:?} -> Registry tweak is currently {}.",
                    id,
                    if is_enabled { "enabled" } else { "disabled" }
                );
                Ok(is_enabled)
            }
            Err(RegistryError::ReadValueError(ref msg))
                if msg.contains("The system cannot find the file specified") =>
            {
                // Assume default state if the value does not exist
                tracing::info!(
                    "{:?} -> Registry tweak is currently disabled (value not found).",
                    id
                );
                Ok(false)
            }
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "{:?} -> Failed to determine if registry tweak is enabled.",
                    id
                );
                Err(e)
            }
        }
    }

    /// Reads the current value of the specified registry key.
    ///
    /// # Returns
    ///
    /// - `Ok(RegistryKeyValue)` with the current value.
    /// - `Err(RegistryError)` if the operation fails.
    pub fn read_current_value(&self, id: TweakId) -> Result<RegistryKeyValue, RegistryError> {
        tracing::trace!("{:?} -> Reading current value of registry tweak.", id);

        // Extract the hive from the key path (e.g., "HKEY_LOCAL_MACHINE")
        let hive = self
            .path
            .split('\\')
            .next()
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.path.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => {
                tracing::error!(
                    hive = %other,
                    "Unsupported registry hive '{}'.",
                    other
                );
                return Err(RegistryError::UnsupportedHive(other.to_string()));
            }
        };

        // Extract the subkey path (everything after the hive)
        let subkey_path = self
            .path
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.path.clone()))?;

        // Attempt to open the subkey with read permissions
        let subkey = match hkey.open_subkey_with_flags(subkey_path, KEY_READ) {
            Ok(key) => {
                tracing::debug!(
                    "{:?} -> Opened registry key '{}' for reading.",
                    id,
                    self.path
                );
                key
            }
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "{:?} -> Failed to open registry key '{}' for reading.",
                    id, self.path
                );
                return Err(RegistryError::KeyOpenError(format!(
                    "Failed to open registry key '{}' for reading: {}",
                    self.path, e
                )));
            }
        };

        // Depending on the expected type, read the value
        match &self.tweak_value {
            RegistryKeyValue::String(_) => {
                let val: String = subkey.get_value(&self.key).map_err(|e| {
                    RegistryError::ReadValueError(format!(
                        "Failed to read string value '{}' in key '{}': {}",
                        self.key, self.path, e
                    ))
                })?;
                tracing::debug!("{:?} -> Read string value '{}' = '{}'.", id, self.key, val);
                Ok(RegistryKeyValue::String(val))
            }
            RegistryKeyValue::Dword(_) => {
                let val: u32 = subkey.get_value(&self.key).map_err(|e| {
                    RegistryError::ReadValueError(format!(
                        "Failed to read DWORD value '{}' in key '{}': {}",
                        self.key, self.path, e
                    ))
                })?;
                tracing::debug!("{:?} -> Read DWORD value '{}' = {}.", id, self.key, val);
                Ok(RegistryKeyValue::Dword(val))
            }
        }
    }

    /// Applies the registry tweak by setting the specified registry value.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(RegistryError)` if the operation fails.
    pub fn apply_registry_tweak(&self, id: TweakId) -> Result<(), RegistryError> {
        tracing::info!("Applying registry tweak '{:?}'.", id);

        // Extract the hive from the key path
        let hive = self
            .path
            .split('\\')
            .next()
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.path.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => {
                tracing::error!(
                    hive = %other,
                    "{:?} -> Unsupported registry hive '{}'.", id,
                    other
                );
                return Err(RegistryError::UnsupportedHive(other.to_string()));
            }
        };

        // Extract the subkey path
        let subkey_path = self
            .path
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.path.clone()))?;

        // Attempt to open the subkey with read and write permissions
        // If it doesn't exist, attempt to create it
        let subkey = match hkey.open_subkey_with_flags(subkey_path, KEY_READ | KEY_WRITE) {
            Ok(key) => {
                tracing::debug!("{:?} -> Opened registry key '{}'.", id, self.path);
                key
            }
            Err(_) => {
                // Subkey does not exist; attempt to create it
                match hkey.create_subkey(subkey_path) {
                    Ok((key, disposition)) => {
                        match disposition {
                            winreg::enums::RegDisposition::REG_CREATED_NEW_KEY => {
                                tracing::info!(
                                    "{:?} -> Created new registry key '{}'.",
                                    id,
                                    self.path
                                );
                            }
                            winreg::enums::RegDisposition::REG_OPENED_EXISTING_KEY => {
                                tracing::debug!(
                                    "{:?} -> Opened existing registry key '{}'.",
                                    id,
                                    self.path
                                );
                            }
                        }
                        key
                    }
                    Err(e) => {
                        tracing::error!(
                            error = ?e,
                            "{:?} -> Failed to create registry key '{}'.", id, self.path
                        );
                        return Err(RegistryError::CreateError(format!(
                            "Failed to create registry key '{}': {}",
                            self.path, e
                        )));
                    }
                }
            }
        };

        // Now, set the registry value based on its type
        match &self.tweak_value {
            RegistryKeyValue::String(val) => {
                subkey.set_value(&self.key, val).map_err(|e| {
                    RegistryError::SetValueError(format!(
                        "Failed to set string value '{}' in key '{}': {}",
                        self.key, self.path, e
                    ))
                })?;
                tracing::info!(
                    tweak_key = %self.path,
                    "{:?} -> Set string value '{}' to '{}'.",
                    id,
                    self.key,
                    val
                );
            }
            RegistryKeyValue::Dword(val) => {
                subkey.set_value(&self.key, val).map_err(|e| {
                    RegistryError::SetValueError(format!(
                        "Failed to set DWORD value '{}' in key '{}': {}",
                        self.key, self.path, e
                    ))
                })?;
                tracing::info!("{:?} -> Set DWORD value '{}' to {}.", id, self.key, val);
            } // Handle other types as needed
        }

        Ok(())
    }

    /// Reverts the registry tweak by restoring the default registry value.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(RegistryError)` if the operation fails.
    pub fn revert_registry_tweak(&self, id: TweakId) -> Result<(), RegistryError> {
        tracing::info!("{:?} -> Reverting registry tweak.", id);

        // Extract the hive from the key path
        let hive = self
            .path
            .split('\\')
            .next()
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.path.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => {
                tracing::error!(
                    hive = %other,
                    "{:?} -> Unsupported registry hive '{}'.", id,
                    other
                );
                return Err(RegistryError::UnsupportedHive(other.to_string()));
            }
        };

        // Extract the subkey path
        let subkey_path = self
            .path
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryError::InvalidKeyFormat(self.path.clone()))?;

        // Open the subkey with write permissions to modify the value
        let subkey = match hkey.open_subkey_with_flags(subkey_path, KEY_WRITE) {
            Ok(key) => {
                tracing::debug!("{:?} -> Opened registry key for writing.", id);
                key
            }
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "{:?} -> Failed to open registry key for writing.", id
                );
                return Err(RegistryError::KeyOpenError(format!(
                    "Failed to open registry key '{}' for writing: {}",
                    self.path, e
                )));
            }
        };

        // Set the registry value back to its default
        match &self.default_value {
            RegistryKeyValue::String(val) => {
                subkey.set_value(&self.key, val).map_err(|e| {
                    RegistryError::SetValueError(format!(
                        "Failed to set string value '{}' in key '{}': {}",
                        self.key, self.path, e
                    ))
                })?;
                tracing::info!(
                    tweak_name = %self.key,
                    tweak_key = %self.path,
                    "{:?} -> Reverted string value '{}' to '{}'.",
                    id,
                    self.key,
                    val
                );
            }
            RegistryKeyValue::Dword(val) => {
                subkey.set_value(&self.key, val).map_err(|e| {
                    RegistryError::SetValueError(format!(
                        "Failed to set DWORD value '{}' in key '{}': {}",
                        self.key, self.path, e
                    ))
                })?;
                tracing::info!(
                    tweak_name = %self.key,
                    tweak_key = %self.path,
                    "{:?} -> Reverted DWORD value '{}' to {}.",
                    id,
                    self.key,
                    val
                );
            } // Handle other types as needed
        }

        Ok(())
    }
}

pub fn enable_large_system_cache() -> RegistryTweak {
    RegistryTweak {
        name: "LargeSystemCache".to_string(),
        description: "Optimizes system memory management by adjusting the LargeSystemCache setting."
            .to_string(),
        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management"
            .to_string(),
        key: "LargeSystemCache".to_string(),
        // Windows will act as a server, optimizing for file sharing and network operations, potentially improving RAM disk performance.
        tweak_value: RegistryKeyValue::Dword(1),
        // Windows will favor foreground applications in terms of memory allocation.
        default_value: RegistryKeyValue::Dword(0),
    }
}

pub fn system_responsiveness() -> RegistryTweak {
    RegistryTweak {
        name: "SystemResponsiveness".to_string(),
        description: "Optimizes system responsiveness by adjusting the SystemResponsiveness setting."
            .to_string(),
        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile"
            .to_string(),
        key: "SystemResponsiveness".to_string(),
        // Windows will favor foreground applications in terms of resource allocation.
        tweak_value: RegistryKeyValue::Dword(0),
        // Windows will favor background services in terms of resource allocation.
        default_value: RegistryKeyValue::Dword(20),
    }
}

pub fn disable_hw_acceleration() -> RegistryTweak {
    RegistryTweak {
        name: "DisableHWAcceleration".to_string(),
        description: "Disables hardware acceleration for the current user.".to_string(),
        path: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics".to_string(),
        key: "DisableHWAcceleration".to_string(),
        // Hardware acceleration is disabled.
        tweak_value: RegistryKeyValue::Dword(1),
        // Hardware acceleration is enabled.
        default_value: RegistryKeyValue::Dword(0),
    }
}

pub fn win32_priority_separation() -> RegistryTweak {
    RegistryTweak {
        name: "Win32PrioritySeparation".to_string(),
        description: "Optimizes system responsiveness by adjusting the Win32PrioritySeparation setting."
            .to_string(),
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583".to_string(),
            key: "Win32PrioritySeparation".to_string(),
            // Foreground applications will receive priority over background services.
            tweak_value: RegistryKeyValue::Dword(26),
            // Background services will receive priority over foreground applications.
            default_value: RegistryKeyValue::Dword(2),
    }
}

pub fn disable_core_parking() -> RegistryTweak {
    RegistryTweak {
        name: "DisableCoreParking".to_string(),
        description: "Disables core parking to improve system performance.".to_string(),
        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583".to_string(),
        key: "ValueMax".to_string(),
        // Core parking is disabled.
        tweak_value: RegistryKeyValue::Dword(0),
        // Core parking is enabled.
        default_value: RegistryKeyValue::Dword(64),
    }
}
