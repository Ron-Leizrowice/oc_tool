use crate::tweaks::{
    registry::{RegistryKeyValue, RegistryModification, RegistryTweak},
    Tweak, TweakCategory, TweakId,
};

pub fn enable_large_system_cache() -> Tweak {
    Tweak::registry_tweak(
        "Large System Cache".to_string(),
        "Optimizes system memory management by adjusting the LargeSystemCache setting."
            .to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::LargeSystemCache,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
                    key: "LargeSystemCache".to_string(),
                    // Windows will act as a server, optimizing for file sharing and network operations, potentially improving RAM disk performance.
                    target_value: RegistryKeyValue::Dword(1),
                    // Windows will favor foreground applications in terms of memory allocation.
                    default_value: Some(RegistryKeyValue::Dword(0)),
                },
            ],
        },
        true,
    )
}

pub fn system_responsiveness() -> Tweak {
    Tweak::registry_tweak(
        "System Responsiveness".to_string(),
        "Optimizes system responsiveness by adjusting the SystemResponsiveness setting."
            .to_string(),
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::SystemResponsiveness,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile".to_string(),
                    key: "SystemResponsiveness".to_string(),
                    // Windows will favor foreground applications in terms of resource allocation.
                    target_value: RegistryKeyValue::Dword(0),
                    // Windows will favor background services in terms of resource allocation.
                    default_value: Some(RegistryKeyValue::Dword(20)),

                },
            ],
        },
        false,
    )
}

pub fn disable_hw_acceleration() -> Tweak {
    Tweak::registry_tweak(
        "Disable Hardware Acceleration".to_string(),
        "Disables hardware acceleration for the current user.".to_string(),
        TweakCategory::Graphics,
        RegistryTweak {
            id: TweakId::DisableHWAcceleration,
            modifications: vec![RegistryModification {
                path: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics".to_string(),
                key: "DisableHWAcceleration".to_string(),
                target_value: RegistryKeyValue::Dword(1),
                default_value: Some(RegistryKeyValue::Dword(0)),
            }],
        },
        false,
    )
}

pub fn win32_priority_separation() -> Tweak {
    Tweak::registry_tweak(
        "Win32PrioritySeparation".to_string(),
        "Optimizes system responsiveness by adjusting the Win32PrioritySeparation setting."
            .to_string(),
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::Win32PrioritySeparation,
            modifications: vec![RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl"
                    .to_string(),
                key: "Win32PrioritySeparation".to_string(),
                // Foreground applications will receive priority over background services.
                target_value: RegistryKeyValue::Dword(26),
                // Background services will receive priority over foreground applications.
                default_value: Some(RegistryKeyValue::Dword(2)),
            }],
        },
        false,
    )
}

pub fn disable_core_parking() -> Tweak {
    Tweak::registry_tweak(
        "Disable Core Parking".to_string(),
        "Disables core parking to improve system performance.".to_string(),
        TweakCategory::Power,
        RegistryTweak {
            id: TweakId::DisableCoreParking,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583".to_string(),
                    key: "ValueMax".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(64)),
                },
            ],
        },
        false,
    )
}

pub fn disable_low_disk_space_checks() -> Tweak {
    Tweak::registry_tweak(
        "Disable Low Disk Space Checks".to_string(),
        "Disables low disk space checks to prevent notifications.".to_string(),
        TweakCategory::Storage,
        RegistryTweak {
            id: TweakId::NoLowDiskSpaceChecks,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer".to_string(),
                    key: "NoLowDiskSpaceChecks".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: Some(RegistryKeyValue::Dword(0)),
                },
            ],
        },
        false,
    )
}

pub fn disable_ntfs_tunnelling() -> Tweak {
    Tweak::registry_tweak(
        "Disable NTFS Tunnelling".to_string(),
        "Disables NTFS tunnelling to improve file system performance.".to_string(),
        TweakCategory::Storage,
        RegistryTweak {
            id: TweakId::DisableNtfsTunnelling,
            modifications: vec![RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem"
                    .to_string(),
                key: "MaximumTunnelEntries".to_string(),
                target_value: RegistryKeyValue::Dword(0),
                default_value: None,
            }],
        },
        false,
    )
}

pub fn distribute_timers() -> Tweak {
    Tweak::registry_tweak(
        "Distribute Timers".to_string(),
        "Enables timer distribution across all cores.".to_string(),
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::DistributeTimers,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "DistributeTimers".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
            ],
        },
        false,
    )
}

pub fn disable_windows_error_reporting() -> Tweak {
    Tweak::registry_tweak(
        "Disable Windows Error Reporting".to_string(),
        "Disables Windows Error Reporting by setting the `Disabled` registry value to `1`. This prevents the system from sending error reports to Microsoft but may hinder troubleshooting.".to_string(),
        TweakCategory::Telemetry,
        RegistryTweak {
            id: TweakId::DisableWindowsErrorReporting,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\Windows Error Reporting".to_string(),
                    key: "Disabled".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: Some(RegistryKeyValue::Dword(0)),
                },
            ],
        },
        false,
    )
}

pub fn dont_verify_random_drivers() -> Tweak {
    Tweak::registry_tweak(
        "Don't Verify Random Drivers".to_string(),
        "Disables random driver verification to improve system performance.".to_string(),
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::DontVerifyRandomDrivers,
            modifications: vec![RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem"
                    .to_string(),
                key: "DontVerifyRandomDrivers".to_string(),
                target_value: RegistryKeyValue::Dword(1),
                default_value: None,
            }],
        },
        false,
    )
}

pub fn disable_driver_paging() -> Tweak {
    Tweak::registry_tweak(
        "Disable Driver Paging".to_string(),
        "Prevents drivers from being paged into virtual memory by setting the `DisablePagingExecutive` registry value to `1`. This can enhance system performance by keeping critical drivers in physical memory but may increase memory usage.".to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::DisableDriverPaging,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Session Manager\\Memory Management".to_string(),
                    key: "DisablePagingExecutive".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
            ],
        },
        false,
    )
}

pub fn disable_prefetcher() -> Tweak {
    Tweak::registry_tweak(
        "Disable Prefetcher".to_string(),
        "Disables the Prefetcher service to improve system performance.".to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::DisablePrefetcher,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters".to_string(),
                    key: "EnablePrefetcher".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(3)),
                },
            ],
        },
        false,
    )
}

pub fn disable_application_telemetry() -> Tweak {
    Tweak::registry_tweak(
        "Disable Application Telemetry".to_string(),
        "Disables Windows Application Telemetry by setting the `AITEnable` registry value to `0`. This reduces the collection of application telemetry data but may limit certain features or diagnostics.".to_string(),
        TweakCategory::Telemetry,
        RegistryTweak {
            id: TweakId::DisableApplicationTelemetry,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows\\AppCompat".to_string(),
                    key: "AITEnable".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: None,
                },
            ],
        },
        false,
    )
}

pub fn thread_dpc_disable() -> Tweak {
    Tweak::registry_tweak(
        "Thread DPC Disable".to_string(),
        "Disables or modifies the handling of Deferred Procedure Calls (DPCs) related to threads by setting the 'ThreadDpcEnable' registry value to 0. This aims to reduce DPC overhead and potentially enhance system responsiveness. However, it may lead to system instability or compatibility issues with certain hardware or drivers.".to_string(),
        TweakCategory::Kernel,
        RegistryTweak {
            id: TweakId::ThreadDpcDisable,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "ThreadDpcEnable".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: None,
                },
            ],
        },
        false,
    )
}

pub fn svc_host_split_threshold() -> Tweak {
    Tweak::registry_tweak(
        "Disable SvcHost Split".to_string(),
        "Adjusts the SvcHost Split Threshold in KB to optimize system performance.".to_string(),
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::SvcHostSplitThreshold,
            modifications: vec![RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control".to_string(),
                key: "SvcHostSplitThresholdInKB".to_string(),
                target_value: RegistryKeyValue::Dword(0x0f000000),
                default_value: None,
            }],
        },
        true,
    )
}

pub fn disable_windows_defender() -> Tweak {
    Tweak::registry_tweak(
        "Disable Windows Defender".to_string(),
        "Disables Windows Defender by setting the `DisableAntiSpyware` registry value to `1`. This prevents Windows Defender from running and may leave your system vulnerable to malware.".to_string(),
        TweakCategory::Security,
        RegistryTweak {
            id: TweakId::DisableWindowsDefender,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows Defender".to_string(),
                    key: "DisableAntiSpyware".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
            ],
        },
        false,
    )
}

pub fn disable_page_file_encryption() -> Tweak {
    Tweak::registry_tweak(
        "Disable Page File Encryption".to_string(),
        "Disables page file encryption to improve system performance.".to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::DisablePageFileEncryption,
            modifications: vec![RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem"
                    .to_string(),
                key: "NtfsEncryptPagingFile".to_string(),
                target_value: RegistryKeyValue::Dword(0),
                default_value: Some(RegistryKeyValue::Dword(1)),
            }],
        },
        true,
    )
}

pub fn disable_intel_tsx() -> Tweak {
    Tweak::registry_tweak(
        "Disable Intel TSX".to_string(),
        "Disables Intel Transactional Synchronization Extensions (TSX) operations to mitigate potential security vulnerabilities.".to_string(),
        TweakCategory::Security,
        RegistryTweak {
            id: TweakId::DisableIntelTSX,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Kernel".to_string(),
                    key: "DisableTsx".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
            ],
        },
        true,
    )
}

pub fn disable_windows_maintenance() -> Tweak {
    Tweak::registry_tweak(
        "Disable Windows Maintenance".to_string(),
        "Disables Windows Maintenance by setting the `MaintenanceDisabled` registry value to `1`. This prevents Windows from performing maintenance tasks, such as software updates, system diagnostics, and security scans.".to_string(),
        TweakCategory::Action,
        RegistryTweak {
            id: TweakId::DisableWindowsMaintenance,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Schedule\\Maintenance".to_string(),
                    key: "MaintenanceDisabled".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
            ],
        },
        false,
    )
}
