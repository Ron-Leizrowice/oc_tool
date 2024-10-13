// src/tweaks/registry_tweaks.rs

use std::sync::{Arc, Mutex};

use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey,
};

use super::{Tweak, TweakCategory};
use crate::tweaks::{method::TweakMethod, TweakId};

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
    pub fn read_current_value(&self, id: TweakId) -> Result<RegistryKeyValue, anyhow::Error> {
        tracing::trace!("{:?} -> Reading current value of registry tweak.", id);

        // Extract the hive from the key path (e.g., "HKEY_LOCAL_MACHINE")
        let hive = self
            .path
            .split('\\')
            .next()
            .ok_or_else(|| anyhow::Error::msg("Failed to extract hive from key path"));

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            Ok("HKEY_LOCAL_MACHINE") => RegKey::predef(HKEY_LOCAL_MACHINE),
            Ok("HKEY_CURRENT_USER") => RegKey::predef(HKEY_CURRENT_USER),
            other => {
                tracing::error!("Unsupported registry hive '{:?}'.", other);
                return Err(anyhow::Error::msg(format!(
                    "Unsupported registry hive '{:?}'.",
                    other
                )));
            }
        };

        // Extract the subkey path (everything after the hive)
        let subkey_path = self
            .path
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| anyhow::Error::msg("Failed to extract subkey path from key path"))?;

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
                return Err(anyhow::Error::from(e));
            }
        };

        // Depending on the expected type, read the value
        match &self.tweak_value {
            RegistryKeyValue::Dword(_) => {
                let val: u32 = subkey.get_value(&self.key).map_err(anyhow::Error::from)?;
                tracing::debug!("{:?} -> Read DWORD value '{}' = {}.", id, self.key, val);
                Ok(RegistryKeyValue::Dword(val))
            }
        }
    }
}

impl TweakMethod for RegistryTweak {
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
    fn initial_state(&self, id: TweakId) -> Result<bool, anyhow::Error> {
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
            Err(_) => {
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
    fn apply(&self, id: TweakId) -> Result<(), anyhow::Error> {
        tracing::info!("Applying registry tweak '{:?}'.", id);

        // Extract the hive from the key path
        let hive = self
            .path
            .split('\\')
            .next()
            .ok_or_else(|| anyhow::Error::msg("Failed to extract hive from key path"))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => {
                tracing::error!("{:?} -> Unsupported registry hive '{}'.", id, other);
                return Err(anyhow::Error::msg(format!(
                    "Unsupported registry hive: {}",
                    other
                )));
            }
        };

        // Extract the subkey path
        let subkey_path = self
            .path
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| anyhow::Error::msg("Failed to extract subkey path from key path"))?;

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
                        return Err(anyhow::Error::msg(format!(
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
                    anyhow::Error::msg(format!(
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
    /// - `Err(anyhow::Error)` if the operation fails.
    fn revert(&self, id: TweakId) -> Result<(), anyhow::Error> {
        tracing::info!("{:?} -> Reverting registry tweak.", id);

        // Extract the hive from the key path
        let hive = self
            .path
            .split('\\')
            .next()
            .ok_or_else(|| anyhow::Error::msg("Failed to extract hive from key path"))?;

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
                return Err(anyhow::Error::msg(format!(
                    "Unsupported registry hive: {}",
                    other
                )));
            }
        };

        // Extract the subkey path
        let subkey_path = self
            .path
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| anyhow::Error::msg("Failed to extract subkey path from key path"))?;

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
                return Err(anyhow::Error::msg(format!(
                    "Failed to open registry key for writing: {}",
                    e
                )));
            }
        };

        match &self.default_value {
            Some(default_val) => {
                // Restore the registry value to its default
                match default_val {
                    RegistryKeyValue::Dword(val) => {
                        subkey.set_value(&self.key, val).map_err(|e| {
                            anyhow::Error::msg(format!(
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
                            return Err(anyhow::Error::msg(format!(
                                "Failed to delete registry value '{}': {}",
                                self.key, e
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
    Tweak::registry_tweak(
        "Large System Cache".to_string(),
        "Optimizes system memory management by adjusting the LargeSystemCache setting."
            .to_string(),
            TweakCategory::Memory,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management"
                .to_string(),
            key: "LargeSystemCache".to_string(),
            // Windows will act as a server, optimizing for file sharing and network operations, potentially improving RAM disk performance.
            tweak_value: RegistryKeyValue::Dword(1),
            // Windows will favor foreground applications in terms of memory allocation.
            default_value: Some(RegistryKeyValue::Dword(0)),
        },
        true,
    )
}

pub fn system_responsiveness() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "System Responsiveness".to_string(),
        "Optimizes system responsiveness by adjusting the SystemResponsiveness setting."
            .to_string(),
            TweakCategory::System,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile"
                .to_string(),
            key: "SystemResponsiveness".to_string(),
            // Windows will favor foreground applications in terms of resource allocation.
            tweak_value: RegistryKeyValue::Dword(0),
            // Windows will favor background services in terms of resource allocation.
            default_value: Some(RegistryKeyValue::Dword(20)),
        },
        false,
    )
}

pub fn disable_hw_acceleration() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Hardware Acceleration".to_string(),
        "Disables hardware acceleration for the current user.".to_string(),
        TweakCategory::Graphics,
        RegistryTweak {
            path: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics".to_string(),
            key: "DisableHWAcceleration".to_string(),
            // Hardware acceleration is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Hardware acceleration is enabled.
            default_value: Some(RegistryKeyValue::Dword(0)),
        },
        false,
    )
}

pub fn win32_priority_separation() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Win32PrioritySeparation".to_string(),
        "Optimizes system responsiveness by adjusting the Win32PrioritySeparation setting."
            .to_string(),
        TweakCategory::System,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl".to_string(),
            key: "Win32PrioritySeparation".to_string(),
            // Foreground applications will receive priority over background services.
            tweak_value: RegistryKeyValue::Dword(26),
            // Background services will receive priority over foreground applications.
            default_value: Some(RegistryKeyValue::Dword(2)),
        },
        false,
    )
}

pub fn disable_core_parking() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Core Parking".to_string(),
        "Disables core parking to improve system performance.".to_string(),
        TweakCategory::Power,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583".to_string(),
            key: "ValueMax".to_string(),
            // Core parking is disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // Core parking is enabled.
            default_value: Some(RegistryKeyValue::Dword(64)),
        },
        false,
    )
}

pub fn disable_low_disk_space_checks() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Low Disk Space Checks".to_string(),
        "Disables low disk space checks to prevent notifications.".to_string(),
        TweakCategory::Storage,
        RegistryTweak {
            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer".to_string(),
            key: "NoLowDiskSpaceChecks".to_string(),
            // Low disk space checks are disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Low disk space checks are enabled.
            default_value: Some(RegistryKeyValue::Dword(0)),
        },
        false,
    )
}

pub fn disable_ntfs_tunnelling() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable NTFS Tunnelling".to_string(),
        "Disables NTFS tunnelling to improve file system performance.".to_string(),
        TweakCategory::Storage,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem".to_string(),
            key: "MaximumTunnelEntries".to_string(),
            // NTFS tunnelling is disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // NTFS tunnelling is enabled.
            default_value: None,
        },
        false,
    )
}

pub fn distribute_timers() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Distribute Timers".to_string(),
        "Enables timer distribution across all cores.".to_string(),
        TweakCategory::System,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel"
                .to_string(),
            key: "DistributeTimers".to_string(),
            // Timer distribution is enabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Timer distribution is disabled.
            default_value: None,
        },
        false,
    )
}

pub fn disable_windows_error_reporting() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Windows Error Reporting".to_string(),
        "Disables Windows Error Reporting by setting the `Disabled` registry value to `1`. This prevents the system from sending error reports to Microsoft but may hinder troubleshooting.".to_string(),
        TweakCategory::Telemetry,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\Windows Error Reporting".to_string(),
            key: "Disabled".to_string(),
            // Windows Error Reporting is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Windows Error Reporting is enabled.
            default_value: Some(RegistryKeyValue::Dword(0)),
        },
        false,
    )
}

pub fn dont_verify_random_drivers() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Don't Verify Random Drivers".to_string(),
        "Disables random driver verification to improve system performance.".to_string(),
        TweakCategory::System,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem".to_string(),
            key: "DontVerifyRandomDrivers".to_string(),
            // Random driver verification is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Random driver verification is enabled.
            default_value: None,
        },
        false,
    )
}

pub fn disable_driver_paging() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Driver Paging".to_string(),
        "Prevents drivers from being paged into virtual memory by setting the `DisablePagingExecutive` registry value to `1`. This can enhance system performance by keeping critical drivers in physical memory but may increase memory usage.".to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
            key: "DisablePagingExecutive".to_string(),
            // Driver paging is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Driver paging is enabled.
            default_value: None,
        },
        false,
    )
}

// HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters create dword EnablePrefetcher=0

pub fn disable_prefetcher() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Prefetcher".to_string(),
        "Disables the Prefetcher service to improve system performance.".to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters".to_string(),
            key: "EnablePrefetcher".to_string(),
            // Prefetcher is disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // Prefetcher is enabled.
            default_value: Some(RegistryKeyValue::Dword(3)),
        },
        false,
    )
}

pub fn disable_application_telemetry() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Application Telemetry".to_string(),
        "Disables Windows Application Telemetry by setting the `AITEnable` registry value to `0`. This reduces the collection of application telemetry data but may limit certain features or diagnostics.".to_string(),
        TweakCategory::Telemetry,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows\\AppCompat".to_string(),
            key: "AITEnable".to_string(),
            // Application telemetry is disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // Application telemetry is enabled.
            default_value: None,
        },
        false,
    )
}

pub fn thread_dpc_disable() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Thread DPC Disable".to_string(),
        "Disables or modifies the handling of Deferred Procedure Calls (DPCs) related to threads by setting the 'ThreadDpcEnable' registry value to 0. This aims to reduce DPC overhead and potentially enhance system responsiveness. However, it may lead to system instability or compatibility issues with certain hardware or drivers.".to_string(),
        TweakCategory::Kernel,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
            key: "ThreadDpcEnable".to_string(),
            // Thread DPCs are disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // Thread DPCs are enabled.
            default_value: None,
        },
        false,
    )
}

pub fn svc_host_split_threshold() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable SvcHost Split".to_string(),
        "Adjusts the SvcHost Split Threshold in KB to optimize system performance.".to_string(),
        TweakCategory::System,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control".to_string(),
            key: "SvcHostSplitThresholdInKB".to_string(),
            tweak_value: RegistryKeyValue::Dword(0x0f000000),
            default_value: None,
        },
        true,
    )
}

pub fn disable_windows_defender() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Windows Defender".to_string(),
        "Disables Windows Defender by setting the `DisableAntiSpyware` registry value to `1`. This prevents Windows Defender from running and may leave your system vulnerable to malware.".to_string(),
        TweakCategory::Security,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows Defender".to_string(),
            key: "DisableAntiSpyware".to_string(),
            // Windows Defender is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Windows Defender is enabled.
            default_value: None,
        },
        false,
    )
}

pub fn disable_page_file_encryption() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Page File Encryption".to_string(),
        "Disables page file encryption to improve system performance.".to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem".to_string(),
            key: "NtfsEncryptPagingFile".to_string(),
            // Page file encryption is disabled.
            tweak_value: RegistryKeyValue::Dword(0),
            // Page file encryption is enabled.
            default_value: Some(RegistryKeyValue::Dword(1)),
        },
        true,
    )
}

pub fn disable_intel_tsx() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Intel TSX".to_string(),
        "Disables Intel Transactional Synchronization Extensions (TSX) operations to mitigate potential security vulnerabilities.".to_string(),
        TweakCategory::Security,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Kernel".to_string(),
            key: "DisableTsx".to_string(),
            // Intel TSX operations are disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Intel TSX operations are enabled.
            default_value: None,
        },
        true,
    )
}

pub fn disable_windows_maintenance() -> Arc<Mutex<Tweak>> {
    Tweak::registry_tweak(
        "Disable Windows Maintenance".to_string(),
        "Disables Windows Maintenance by setting the `MaintenanceDisabled` registry value to `1`. This prevents Windows from performing maintenance tasks, such as software updates, system diagnostics, and security scans.".to_string(),
        TweakCategory::Action,
        RegistryTweak {
            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Schedule\\Maintenance".to_string(),
            key: "MaintenanceDisabled".to_string(),
            // Windows Maintenance is disabled.
            tweak_value: RegistryKeyValue::Dword(1),
            // Windows Maintenance is enabled.
            default_value: None,
        },
        false,
    )
}
