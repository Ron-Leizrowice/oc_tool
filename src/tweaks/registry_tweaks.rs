// src/tweaks/registry_tweaks.rs

use std::sync::{Arc, Mutex};

use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey,
};

use super::{Tweak, TweakCategory, TweakMethod};
use crate::{errors::RegistryError, tweaks::TweakId, widgets::TweakWidget};

/// Represents a registry tweak, including the registry key, value name, desired value, and default value.
/// If `default_value` is `None`, the tweak is considered enabled if the registry value exists.
/// Reverting such a tweak involves deleting the registry value.
#[derive(Clone, Debug)]
pub struct RegistryTweak {
    /// Full path of the registry key (e.g., "HKEY_LOCAL_MACHINE\\Software\\...").
    pub path: String,
    /// Name of the registry value to modify.
    pub key: String,
    /// The value to set when applying the tweak.
    pub tweak_value: RegistryKeyValue,
    /// The default value to revert to when undoing the tweak.
    /// If `None`, reverting deletes the registry value.
    pub default_value: Option<RegistryKeyValue>,
}

/// Enumeration of supported registry key value types.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RegistryKeyValue {
    Dword(u32),
    // Add other types as needed (e.g., Qword, Binary, etc.)
}

impl RegistryTweak {
    /// Checks if the tweak is currently enabled.
    ///
    /// - If `default_value` is `Some(value)`, the tweak is enabled if the current registry value equals `value`.
    /// - If `default_value` is `None`, the tweak is enabled if the registry value exists.
    ///
    /// # Parameters
    ///
    /// - `id`: The unique identifier for the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the tweak is enabled.
    /// - `Ok(false)` if the tweak is disabled.
    /// - `Err(RegistryError)` if an error occurs while reading the registry.
    pub fn is_registry_tweak_enabled(&self, id: TweakId) -> Result<bool, RegistryError> {
        tracing::info!("{:?} -> Determining if registry tweak is enabled.", id);
        match self.read_current_value(id) {
            Ok(current_value) => {
                match &self.default_value {
                    Some(default_val) => {
                        let is_enabled = current_value != *default_val;
                        tracing::info!(
                            "{:?} -> Registry tweak is currently {}.",
                            id,
                            if is_enabled { "enabled" } else { "disabled" }
                        );
                        Ok(is_enabled)
                    }
                    None => {
                        // If default_value is None, the tweak is enabled if the key exists
                        tracing::info!(
                            "{:?} -> Registry tweak is currently enabled (value exists).",
                            id
                        );
                        Ok(true)
                    }
                }
            }
            Err(RegistryError::ReadValueError(ref msg))
                if msg.contains("The system cannot find the file specified") =>
            {
                match &self.default_value {
                    Some(_) => {
                        // With a default value, absence means tweak is disabled
                        tracing::info!(
                            "{:?} -> Registry tweak is currently disabled (value not found).",
                            id
                        );
                        Ok(false)
                    }
                    None => {
                        // Without a default value, absence means tweak is disabled
                        tracing::info!(
                            "{:?} -> Registry tweak is currently disabled (value not found).",
                            id
                        );
                        Ok(false)
                    }
                }
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
    /// # Parameters
    ///
    /// - `id`: The unique identifier for the tweak.
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
    /// # Parameters
    ///
    /// - `id`: The unique identifier for the tweak.
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

    /// Reverts the registry tweak by restoring the default registry value or deleting it if no default is provided.
    ///
    /// # Parameters
    ///
    /// - `id`: The unique identifier for the tweak.
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

        match &self.default_value {
            Some(default_val) => {
                // Restore the registry value to its default
                match default_val {
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
            }
            None => {
                // If no default value, delete the registry value
                match subkey.delete_value(&self.key) {
                    Ok(_) => {
                        tracing::info!(
                            tweak_name = %self.key,
                            tweak_key = %self.path,
                            "{:?} -> Deleted registry value '{}'.",
                            id,
                            self.key
                        );
                    }
                    Err(e) => {
                        // If the value does not exist, it's already in the default state
                        if e.kind() == std::io::ErrorKind::NotFound {
                            tracing::info!(
                                tweak_name = %self.key,
                                tweak_key = %self.path,
                                "{:?} -> Registry value '{}' does not exist. No action needed.",
                                id,
                                self.key
                            );
                            return Ok(());
                        } else {
                            tracing::error!(
                                error = ?e,
                                "{:?} -> Failed to delete registry value '{}'.",
                                id,
                                self.key
                            );
                            return Err(RegistryError::DeleteValueError(format!(
                                "Failed to delete registry value '{}' in key '{}': {}",
                                self.key, self.path, e
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn enable_large_system_cache() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::LargeSystemCache,
        "Large System Cache".to_string(),
        "Optimizes system memory management by adjusting the LargeSystemCache setting."
            .to_string(),
            TweakCategory::Memory,
            vec![
                "https://archive.arstechnica.com/tweak/nt/cache.html"
                    .to_string(),
            ],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management"
                .to_string(),
            key: "LargeSystemCache".to_string(),
            // Windows will act as a server, optimizing for file sharing and network operations, potentially improving RAM disk performance.
            tweak_value: RegistryKeyValue::Dword(1),
            // Windows will favor foreground applications in terms of memory allocation.
            default_value: Some(RegistryKeyValue::Dword(0)),
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn system_responsiveness() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::SystemResponsiveness,
        "System Responsiveness".to_string(),
        "Optimizes system responsiveness by adjusting the SystemResponsiveness setting."
            .to_string(),
            TweakCategory::System,
            vec![
                "https://www.back2gaming.com/guides/how-to-tweak-windows-10-for-gaming/"
                    .to_string(),
            ],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile"
                .to_string(),
            key: "SystemResponsiveness".to_string(),
            // Windows will favor foreground applications in terms of resource allocation.
            tweak_value: RegistryKeyValue::Dword(0),
            // Windows will favor background services in terms of resource allocation.
            default_value: Some(RegistryKeyValue::Dword(20)),
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_hw_acceleration() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableHWAcceleration,
        "Disable Hardware Acceleration".to_string(),
        "Disables hardware acceleration for the current user.".to_string(),
        TweakCategory::Graphics,
        vec!["https://www.majorgeeks.com/content/page/how_to_disable_or_adjust_hardware_acceleration_in_windows.html#:~:text=Press%20the%20Windows%20Key%20%2B%20S,GPU%20scheduling%20on%20or%20off.".to_string()],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics".to_string(),
            key: "DisableHWAcceleration".to_string(),
            // Hardware acceleration is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Hardware acceleration is enabled.
            default_value: Some(RegistryKeyValue::Dword(0)),
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn win32_priority_separation() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::Win32PrioritySeparation,
        "Win32PrioritySeparation".to_string(),
        "Optimizes system responsiveness by adjusting the Win32PrioritySeparation setting."
            .to_string(),
        TweakCategory::System,
        vec![
            "https://docs.google.com/document/d/1c2-lUJq74wuYK1WrA_bIvgb89dUN0sj8-hO3vqmrau4/edit"
                .to_string(),
        ],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl".to_string(),
            key: "Win32PrioritySeparation".to_string(),
            // Foreground applications will receive priority over background services.
            tweak_value: RegistryKeyValue::Dword(26),
            // Background services will receive priority over foreground applications.
            default_value: Some(RegistryKeyValue::Dword(2)),
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_core_parking() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableCoreParking,
        "Disable Core Parking".to_string(),
        "Disables core parking to improve system performance.".to_string(),
        TweakCategory::Power,
        vec!["https://www.overclock.net/threads/core-parking-in-windows-disable-for-more-performance.1544554/".to_string()],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583".to_string(),
            key: "ValueMax".to_string(),
            // Core parking is disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // Core parking is enabled.
            default_value: Some(RegistryKeyValue::Dword(64)),
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_low_disk_space_checks() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::NoLowDiskSpaceChecks,
        "Disable Low Disk Space Checks".to_string(),
        "Disables low disk space checks to prevent notifications.".to_string(),
        TweakCategory::Storage,
        vec!["https://www.howtogeek.com/349523/how-to-disable-the-low-disk-space-warning-on-windows/".to_string()],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer".to_string(),
            key: "NoLowDiskSpaceChecks".to_string(),
            // Low disk space checks are disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Low disk space checks are enabled.
            default_value: Some(RegistryKeyValue::Dword(0)),
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_ntfs_tunnelling() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableNtfsTunnelling,
        "Disable NTFS Tunnelling".to_string(),
        "Disables NTFS tunnelling to improve file system performance.".to_string(),
        TweakCategory::Storage,
        vec!["https://tweaks.com/windows/37011/optimise-ntfs/".to_string()],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem".to_string(),
            key: "MaximumTunnelEntries".to_string(),
            // NTFS tunnelling is disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // NTFS tunnelling is enabled.
            default_value: None,
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn distribute_timers() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DistributeTimers,
        "Distribute Timers".to_string(),
        "Enables timer distribution across all cores.".to_string(),
        TweakCategory::System,
        vec![
            "https://sites.google.com/view/melodystweaks/misconceptions-about-timers-hpet-tsc-pmt"
                .to_string(),
        ],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel"
                .to_string(),
            key: "DistributeTimers".to_string(),
            // Timer distribution is enabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Timer distribution is disabled.
            default_value: None,
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_windows_error_reporting() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableWindowsErrorReporting,
        "Disable Windows Error Reporting".to_string(),
        "Disables Windows Error Reporting by setting the `Disabled` registry value to `1`. This prevents the system from sending error reports to Microsoft but may hinder troubleshooting.".to_string(),
        TweakCategory::Telemetry,
        vec!["https://www.makeuseof.com/windows-disable-error-reporting/".to_string()],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\Windows Error Reporting".to_string(),
            key: "Disabled".to_string(),
            // Windows Error Reporting is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Windows Error Reporting is enabled.
            default_value: Some(RegistryKeyValue::Dword(0)),
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn dont_verify_random_drivers() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DontVerifyRandomDrivers,
        "Don't Verify Random Drivers".to_string(),
        "Disables random driver verification to improve system performance.".to_string(),
        TweakCategory::System,
        vec![
            "https://maxcheaters.com/topic/127491-counter-strike-improve-computer-performance/"
                .to_string(),
        ],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem".to_string(),
            key: "DontVerifyRandomDrivers".to_string(),
            // Random driver verification is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Random driver verification is enabled.
            default_value: None,
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_driver_paging() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableDriverPaging,
        "Disable Driver Paging".to_string(),
        "Prevents drivers from being paged into virtual memory by setting the `DisablePagingExecutive` registry value to `1`. This can enhance system performance by keeping critical drivers in physical memory but may increase memory usage.".to_string(),
        TweakCategory::Memory,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
            key: "DisablePagingExecutive".to_string(),
            // Driver paging is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Driver paging is enabled.
            default_value: None,
        }),
        false,
        TweakWidget::Switch,
    )
}

// HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters create dword EnablePrefetcher=0

pub fn disable_prefetcher() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisablePrefetcher,
        "Disable Prefetcher".to_string(),
        "Disables the Prefetcher service to improve system performance.".to_string(),
        TweakCategory::Memory,
        vec!["https://www.tenforums.com/tutorials/82016-enable-disable-prefetch-windows-10-a.html".to_string()],
        TweakMethod::Registry(RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters".to_string(),
            key: "EnablePrefetcher".to_string(),
            // Prefetcher is disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // Prefetcher is enabled.
            default_value: Some(RegistryKeyValue::Dword(3)),
        }),
        false,
        TweakWidget::Switch,
    )
}
