// src/tweaks/registry_tweaks.rs

use druid::{Data, Lens};
use once_cell::sync::Lazy;
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey,
};

use super::TweakMethod;
use crate::{errors::RegistryTweakError, models::Tweak, ui::widgets::WidgetType};

// Subclass for Registry tweaks
#[derive(Clone, Data, Lens, Debug)]
pub struct RegistryTweak {
    pub key: String,
    pub name: String,
    pub value: RegistryKeyValue,
    pub default_value: RegistryKeyValue,
}

#[derive(Clone, Data, PartialEq, Eq, Debug)]
pub enum RegistryKeyValue {
    String(String),
    Dword(u32),
    // Add other types as needed (e.g., Qword, Binary, etc.)
}

impl RegistryTweak {
    // Function to read the current registry value
    pub fn read_current_value(&self) -> Result<RegistryKeyValue, RegistryTweakError> {
        // Extract the hive from the key path
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryTweakError::UnsupportedHive(other.to_string())),
        };

        // Extract the subkey path (everything after the hive)
        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        // Attempt to open the subkey with read permissions
        let subkey = hkey
            .open_subkey_with_flags(subkey_path, KEY_READ)
            .map_err(|e| {
                RegistryTweakError::KeyOpenError(format!(
                    "Failed to open registry key '{}': {}",
                    self.key, e
                ))
            })?;

        // Depending on the expected type, read the value
        match &self.value {
            RegistryKeyValue::String(_) => {
                let val: String = subkey.get_value(&self.name).map_err(|e| {
                    RegistryTweakError::ReadValueError(format!(
                        "Failed to read string value '{:.?}': {:.?}",
                        self.value, e
                    ))
                })?;
                Ok(RegistryKeyValue::String(val))
            }
            RegistryKeyValue::Dword(_) => {
                let val: u32 = subkey.get_value(&self.name).map_err(|e| {
                    RegistryTweakError::ReadValueError(format!(
                        "Failed to read DWORD value '{:.?}': {:.?}",
                        self.value, e
                    ))
                })?;
                Ok(RegistryKeyValue::Dword(val))
            }
        }
    }

    pub fn apply_registry_tweak(&self) -> Result<(), RegistryTweakError> {
        // Extract the hive from the key path
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryTweakError::UnsupportedHive(other.to_string())),
        };

        // Extract the subkey path (everything after the hive)
        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        // Attempt to open the subkey with read and write permissions
        // If it doesn't exist, create it
        let subkey = match hkey.open_subkey_with_flags(subkey_path, KEY_READ | KEY_WRITE) {
            Ok(key) => key, // Subkey exists and is opened successfully
            Err(_) => {
                // Subkey does not exist; attempt to create it
                match hkey.create_subkey(subkey_path) {
                    Ok((key, disposition)) => {
                        // Log whether the key was created or already existed
                        match disposition {
                            winreg::enums::RegDisposition::REG_CREATED_NEW_KEY => {
                                println!("Created new registry key: {}", self.key);
                            }
                            winreg::enums::RegDisposition::REG_OPENED_EXISTING_KEY => {
                                println!("Opened existing registry key: {}", self.key);
                            }
                        }
                        key
                    }
                    Err(e) => {
                        return Err(RegistryTweakError::CreateError(format!(
                            "Failed to create registry key '{:?}': {:?}",
                            self.key, e
                        )))
                    }
                }
            }
        };

        // Now, set the registry value based on its type
        match &self.value {
            RegistryKeyValue::String(val) => {
                subkey.set_value(&self.name, val).map_err(|e| {
                    RegistryTweakError::SetValueError(format!(
                        "Failed to set string value '{:?}' in key '{:?}': {:?}",
                        self.name, self.key, e
                    ))
                })?;
                println!(
                    "Set string value '{:.?}' to '{:.?}' in key '{:.?}'",
                    self.value, val, self.key
                );
            }
            RegistryKeyValue::Dword(val) => {
                subkey.set_value(&self.name, val).map_err(|e| {
                    RegistryTweakError::SetValueError(format!(
                        "Failed to set DWORD value '{:?}' in key '{:?}': {:?}",
                        self.name, self.key, e
                    ))
                })?;
                println!(
                    "Set DWORD value '{:.?}' to '{:.?}' in key '{:.?}'",
                    self.value, val, self.key
                );
            } // Handle other types as needed
        }

        Ok(())
    }

    pub fn revert_registry_tweak(&self) -> Result<(), RegistryTweakError> {
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryTweakError::UnsupportedHive(other.to_string())),
        };

        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        let subkey = hkey
            .open_subkey_with_flags(subkey_path, KEY_WRITE)
            .map_err(|e| {
                RegistryTweakError::KeyOpenError(format!(
                    "Failed to open registry key '{}': {}",
                    self.key, e
                ))
            })?;

        match &self.default_value {
            RegistryKeyValue::String(val) => subkey
                .set_value(&self.name, val)
                .map_err(|e| RegistryTweakError::SetValueError(e.to_string())),
            RegistryKeyValue::Dword(val) => subkey
                .set_value(&self.name, val)
                .map_err(|e| RegistryTweakError::SetValueError(e.to_string())),
            // Handle other types as needed
        }
    }
}

pub static LARGE_SYSTEM_CACHE: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
    id: 0,
    name: "LargeSystemCache".to_string(),
    description: "Optimizes system memory management by adjusting the LargeSystemCache setting.".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
        name: "LargeSystemCache".to_string(),
        value: RegistryKeyValue::Dword(1),
        default_value: RegistryKeyValue::Dword(0),
    }),
    requires_restart: false,
    applying: false,
}
});

pub static SYSTEM_RESPONSIVENESS: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
    id: 0,
    name: "SystemResponsiveness".to_string(),
    description: "Optimizes system responsiveness by adjusting the SystemResponsiveness setting.".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile".to_string(),
        name: "SystemResponsiveness".to_string(),
        value: RegistryKeyValue::Dword(0),
        default_value: RegistryKeyValue::Dword(20),
    }),
    requires_restart: false,
    applying: false,
}
});

pub static DISABLE_HW_ACCELERATION: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "DisableHWAcceleration".to_string(),
    description: "Disables hardware acceleration for the current user.".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics".to_string(),
        name: "DisableHWAcceleration".to_string(),
        value: RegistryKeyValue::Dword(1),
        default_value: RegistryKeyValue::Dword(0),
    }),
    requires_restart: false,
    applying: false,
});

pub static WIN_32_PRIORITY_SEPARATION: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "Win32PrioritySeparation".to_string(),
    description:
        "Optimizes system responsiveness by adjusting the Win32PrioritySeparation setting."
            .to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\PriorityControl".to_string(),
        name: "Win32PrioritySeparation".to_string(),
        value: RegistryKeyValue::Dword(26),
        default_value: RegistryKeyValue::Dword(2),
    }),
    requires_restart: false,
    applying: false,
});

pub static DISABLE_LOW_DISK_CHECK: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "DisableLowDiskCheck".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    description: "Disables the low disk space check for the current user.".to_string(),
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer"
            .to_string(),
        name: "NoLowDiskSpaceChecks".to_string(),
        value: RegistryKeyValue::Dword(1),
        default_value: RegistryKeyValue::Dword(0),
    }),
    requires_restart: false,
    applying: false,
});

pub static DISABLE_CORE_PARKING: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
    id: 0,
    name: "DisableCoreParking".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    description: "Disables core parking to improve system performance.".to_string(),
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583".to_string(),
        name: "ValueMax".to_string(),
        value: RegistryKeyValue::Dword(0),
        default_value: RegistryKeyValue::Dword(64),
    }),
    requires_restart: true,
    applying: false,
}
});
