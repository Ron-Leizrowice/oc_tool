//  src/tweaks/registry/mod.rs

use indexmap::IndexMap;
use kernel::{additional_kernel_worker_threads, alchemy_kernel_tweak, thread_dpc_disable};
use method::{RegistryModification, RegistryTweak};

use super::{Tweak, TweakCategory, TweakOption};
use crate::{tweaks::TweakId, utils::registry::RegistryKeyValue};

mod kernel;
pub mod method;

pub fn all_registry_tweaks<'a>() -> Vec<(TweakId, Tweak<'a>)> {
    vec![
        (TweakId::LargeSystemCache, enable_large_system_cache()),
        (TweakId::SystemResponsiveness, system_responsiveness()),
        (TweakId::DisableHWAcceleration, disable_hw_acceleration()),
        (
            TweakId::Win32PrioritySeparation,
            win32_priority_separation(),
        ),
        (TweakId::DisableCoreParking, disable_core_parking()),
        (
            TweakId::NoLowDiskSpaceChecks,
            disable_low_disk_space_checks(),
        ),
        (
            TweakId::DisableWindowsErrorReporting,
            disable_windows_error_reporting(),
        ),
        (
            TweakId::DontVerifyRandomDrivers,
            dont_verify_random_drivers(),
        ),
        (TweakId::DisableDriverPaging, disable_driver_paging()),
        (TweakId::DisablePrefetcher, configure_prefetcher_service()),
        (
            TweakId::DisableApplicationTelemetry,
            disable_application_telemetry(),
        ),
        (TweakId::ThreadDpcDisable, thread_dpc_disable()),
        (TweakId::SvcHostSplitThreshold, svc_host_split_threshold()),
        (TweakId::DisableWindowsDefender, disable_windows_defender()),
        (
            TweakId::DisablePageFileEncryption,
            disable_page_file_encryption(),
        ),
        (TweakId::DisableIntelTSX, disable_intel_tsx()),
        (
            TweakId::DisableWindowsMaintenance,
            disable_windows_maintenance(),
        ),
        (
            TweakId::AdditionalKernelWorkerThreads,
            additional_kernel_worker_threads(),
        ),
        (
            TweakId::SpeculativeExecutionMitigations,
            disable_speculative_execution_mitigations(),
        ),
        (
            TweakId::HighPerformanceVisualSettings,
            high_performance_visual_settings(),
        ),
        (TweakId::SplitLargeCaches, split_large_caches()),
        (
            TweakId::DisableProtectedServices,
            disable_protected_services(),
        ),
        (TweakId::DisablePagingCombining, disable_paging_combining()),
        (
            TweakId::DisableSecurityAccountsManager,
            disable_security_accounts_manager(),
        ),
        (TweakId::EnableMcsss, enable_mcsss()),
        (TweakId::AlchemyKernelTweak, alchemy_kernel_tweak()),
    ]
}

pub fn enable_large_system_cache<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Large System Cache",
        "Controls the system's memory management strategy between desktop/application optimization or server/system service optimization. When enabled (1), the system maintains more data in RAM by aggressively caching file system data and delaying write operations, favoring system services and background tasks. When disabled (0), the system reduces RAM usage for file caching to prioritize foreground application responsiveness. Typically beneficial to enable on systems with large RAM (>1GB) running server workloads or data-intensive background tasks, while desktop systems usually perform better with this disabled.",
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::LargeSystemCache,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                        key: "LargeSystemCache",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                        key: "LargeSystemCache",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        true, // requires reboot
    )
}

pub fn system_responsiveness<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "System Responsiveness",
        "Controls CPU time allocation between foreground applications and background processes. Values represent the percentage of CPU time reserved for background tasks:\n\n\
        • 0% (Disabled): Maximum foreground priority. Provides highest responsiveness for active applications and games, but may cause stuttering in background operations like downloads or updates.\n\
        • 20% (Default): Windows' balanced setting. Reserves one-fifth of CPU time for background tasks while maintaining good foreground responsiveness.\n\
        • 100%: Equal priority for all processes. Ensures smooth background operations but may impact foreground application performance.",
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::SystemResponsiveness,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Option("Default".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
                        key: "SystemResponsiveness",
                        value: RegistryKeyValue::Dword(20),
                    }],
                ),
                (
                    TweakOption::Option("0".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
                        key: "SystemResponsiveness",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Option("100".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
                        key: "SystemResponsiveness",
                        value: RegistryKeyValue::Dword(100),
                    }],
                ),
            ]),
        },
        true, // requires reboot
    )
}

pub fn disable_hw_acceleration<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Hardware Acceleration",
        "Controls Windows' DirectX-based hardware acceleration for the Windows Presentation Foundation (WPF) graphics system. When enabled, graphics operations are offloaded to the GPU for better performance.",
        TweakCategory::Graphics,
        RegistryTweak {
            id: TweakId::DisableHWAcceleration,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics",
                        key: "DisableHWAcceleration",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics",
                        key: "DisableHWAcceleration",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        false, // does not require reboot
    )
}

pub fn win32_priority_separation<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Win32 Priority Separation",
        "Controls how Windows balances processor time between foreground and background processes. The value is a combination of three settings:\n\n\
        • 2A = Short, Fixed, High foreground boost.\n\
        • 29 = Short, Fixed, Medium foreground boost.\n\
        • 28 = Short, Fixed, No foreground boost.\n\
        \n\
        • 26 = Short, Variable , High foreground boost.\n\
        • 25 = Short, Variable , Medium foreground boost.\n\
        • 24 = Short, Variable , No foreground boost.\n\
        \n\
        • 1A = Long, Fixed, High foreground boost.\n\
        • 19 = Long, Fixed, Medium foreground boost.\n\
        • 18 = Long, Fixed, No foreground boost.\n\
        \n\
        • 16 = Long, Variable, High foreground boost.\n\
        • 15 = Long, Variable, Medium foreground boost.\n\
        • 14 = Long, Variable, No foreground boost.",
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::Win32PrioritySeparation,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Option("Default".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(2)
                    }],
                ),
                (
                    TweakOption::Option("2A".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x2A),
                    }],
                ),
                (
                    TweakOption::Option("29".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x29),
                    }],
                ),
                (
                    TweakOption::Option("28".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x28),
                    }],
                ),
                (
                    TweakOption::Option("26".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x26),
                    }],
                ),
                (
                    TweakOption::Option("25".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x25),
                    }],
                ),
                (
                    TweakOption::Option("24".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x24),
                    }],
                ),
                (
                    TweakOption::Option("1A".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x1A),
                    }],
                ),
                (
                    TweakOption::Option("19".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x19),
                    }],
                ),
                (
                    TweakOption::Option("18".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x18),
                    }],
                ),
                (
                    TweakOption::Option("16".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x16),
                    }],
                ),
                (
                    TweakOption::Option("15".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x15),
                    }],
                ),
                (
                    TweakOption::Option("14".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        value: RegistryKeyValue::Dword(0x14),
                    }],
                ),


            ]),
        },
        false, // does not require reboot
    )
}

pub fn disable_core_parking<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Core Parking",
        "Controls the processor core parking behavior, which allows Windows to suspend inactive CPU cores to save power. The value represents the maximum percentage of cores that can be parked:\n\
        • 0% (Disabled): All cores remain active, providing maximum performance at the cost of higher power consumption\n\
        • 64% (Default): Up to 64% of cores can be parked when idle\n\
        Disabling core parking can reduce latency spikes caused by core wake-up delays, beneficial for gaming and latency-sensitive applications. However, it increases power consumption and heat generation.",
        TweakCategory::Power,
        RegistryTweak {
            id: TweakId::DisableCoreParking,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583",
                        key: "ValueMax",
                        value: RegistryKeyValue::Dword(64),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583",
                        key: "ValueMax",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
            ]),
        },
        true, // requires reboot
    )
}

pub fn disable_low_disk_space_checks<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Low Disk Space Checks",
        "Controls Windows' automatic disk space monitoring and notification system. When enabled, Windows monitors free space on all drives and shows warnings when space is low. Disabling removes these notifications but also prevents Windows from automatically cleaning up temporary files when disk space is low. Note: This doesn't affect Windows' ability to write to disks; it only controls the monitoring and notification system. Consider keeping enabled on systems with limited storage.",
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::NoLowDiskSpaceChecks,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer",
                        key: "NoLowDiskSpaceChecks",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer",
                        key: "NoLowDiskSpaceChecks",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        false, // does not require reboot
    )
}

pub fn disable_windows_error_reporting<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Windows Error Reporting",
        "Controls Windows' error reporting system that sends crash and diagnostic data to Microsoft. When enabled, Windows collects and sends error reports to help identify and fix software issues. Disabling prevents all error reporting, which:\n\
        • Increases privacy by preventing diagnostic data transmission\n\
        • Eliminates the delay caused by error report generation after crashes\n\
        • May make troubleshooting harder as no local error reports are generated\n\
        • Prevents automatic notification of fixes for known crashes\n\
        Consider keeping enabled on development machines where crash diagnostics are valuable.",
        TweakCategory::Telemetry,
        RegistryTweak {
            id: TweakId::DisableWindowsErrorReporting,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\Windows Error Reporting",
                        key: "Disabled",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\Windows Error Reporting",
                        key: "Disabled",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        false, // does not require reboot
    )
}

pub fn dont_verify_random_drivers<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Disable Driver Verification",
        "Controls Windows' random driver verification system which periodically verifies the integrity of device drivers.",
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::DontVerifyRandomDrivers,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem",
                        key: "DontVerifyRandomDrivers",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem",
                        key: "DontVerifyRandomDrivers",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        true, // requires reboot
    )
}

pub fn disable_driver_paging<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Driver Paging",
        "Controls whether Windows can page kernel-mode drivers and system code to disk. When disabled:\n\
        • Forces all drivers and system code to remain in physical RAM\n\
        • Can improve system responsiveness by preventing driver code from being paged out\n\
        • Significantly increases RAM usage as drivers cannot be swapped",
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::DisableDriverPaging,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Session Manager\\Memory Management",
                        key: "DisablePagingExecutive",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Session Manager\\Memory Management",
                        key: "DisablePagingExecutive",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        true, // requires reboot
    )
}

pub fn configure_prefetcher_service<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Configure Prefetcher Service",
        "Controls Windows' Prefetcher service that manages application and boot file caching. Available modes:\n\
        • 0 (Disabled): No prefetching of applications or boot files\n\
        • 1 (Application): Only prefetch application files\n\
        • 2 (Boot): Only prefetch boot files\n\
        • 3 (Default): Prefetch both application and boot files\n\
        Disabling can improve performance on SSDs and reduce disk writes.",
        TweakCategory::Services,
        RegistryTweak {
            id: TweakId::DisablePrefetcher,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Option("Default".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters",
                        key: "EnablePrefetcher",
                        value: RegistryKeyValue::Dword(3),
                    }],
                ),
                (
                    TweakOption::Option("Boot Only".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters",
                        key: "EnablePrefetcher",
                        value: RegistryKeyValue::Dword(2),
                    }],
                ),
                (
                    TweakOption::Option("Application Only".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters",
                        key: "EnablePrefetcher",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
                (
                    TweakOption::Option("Disabled".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters",
                        key: "EnablePrefetcher",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),

            ]),
        },
        true, // requires reboot
    )
}

pub fn disable_application_telemetry<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Application Telemetry",
        "Controls Windows' Application Insights Telemetry system that collects application usage and performance data. When disabled:\n\
        • Prevents collection and transmission of application usage patterns\n\
        • Stops reporting of application compatibility issues to Microsoft\n\
        • May reduce effectiveness of application compatibility features\n\
        • Improves privacy and slightly reduces system overhead\n\
        Note: This setting specifically affects application compatibility telemetry and is separate from other Windows telemetry settings.",
        TweakCategory::Telemetry,
        RegistryTweak {
            id: TweakId::DisableApplicationTelemetry,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows\\AppCompat",
                        key: "AITEnable",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows\\AppCompat",
                        key: "AITEnable",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
            ]),
        },
        false, // does not require reboot
    )
}

pub fn svc_host_split_threshold<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Service Host Grouping Threshold",
        "Controls how Windows groups services into shared Service Host (svchost.exe) processes based on system RAM. The threshold determines when services get their own process:\n\
        • Default: Windows automatically groups services based on memory usage\n\
        • No Split: All services run in the same svchost.exe process\n\
        • Max Split: Each service runs in its own svchost.exe process\n\
        Higher values increase service isolation, improving stability and security but using more memory. Lower values reduce memory usage but increase the impact of service crashes. Warning: Very high values may significantly increase RAM usage.",
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::SvcHostSplitThreshold,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Option("Default".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control",
                        key: "SvcHostSplitThresholdInKB",
                        value: RegistryKeyValue::Dword(0x00380000),
                    }],
                ),
                (
                    TweakOption::Option("No Split".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control",
                        key: "SvcHostSplitThresholdInKB",
                        value: RegistryKeyValue::Dword(0x33554432),
                    }],
                ),
                (
                    TweakOption::Option("Max Split".to_string()),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control",
                        key: "SvcHostSplitThresholdInKB",
                        value: RegistryKeyValue::Dword(0x0),
                    }],
                ),
            ]),
        },
        true, // requires reboot
    )
}

pub fn disable_windows_defender<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Disable Windows Defender",
        "Controls Windows Defender's real-time protection and monitoring capabilities.",
        TweakCategory::Security,
        RegistryTweak {
            id: TweakId::DisableWindowsDefender,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows Defender",
                        key: "DisableAntiSpyware",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows Defender",
                        key: "DisableAntiSpyware",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        true, // Requires reboot
    )
}

pub fn disable_page_file_encryption<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Disable Page File Encryption",
        "Controls NTFS page file encryption which protects sensitive data when written to disk. When disabled:\n\
        • Reduces disk I/O overhead from encryption/decryption\n\
        • May slightly improve system performance",
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::DisablePageFileEncryption,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem",
                        key: "NtfsEncryptPagingFile",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem",
                        key: "NtfsEncryptPagingFile",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
            ]),
        },
        true, // Requires reboot 
    )
}

pub fn disable_intel_tsx<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Intel TSX",
        "Controls Intel Transactional Synchronization Extensions (TSX), a CPU feature for hardware transactional memory. When disabled:\n\
        • Mitigates potential security vulnerabilities (MDS, Zombieload)\n\
        • Prevents speculative execution attacks via TSX\n\
        • May reduce performance in heavily multi-threaded applications\n\
        • Affects only Intel CPUs with TSX support\n\
        Note: Modern Intel CPUs often have TSX disabled by default via microcode. This setting ensures it remains disabled even if microcode is updated.",
        TweakCategory::Security,
        RegistryTweak {
            id: TweakId::DisableIntelTSX,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Kernel",
                        key: "DisableTsx",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Kernel",
                        key: "DisableTsx",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        true, // Requires reboot for CPU feature changes
    )
}

pub fn disable_windows_maintenance<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Disable Windows Maintenance",
        "Controls Windows' automated maintenance tasks that run during system idle time. These tasks include:\n\
        • Disk defragmentation and optimization\n\
        • System diagnostics and error reporting\n\
        • Windows Update maintenance and cache cleanup\n\
        • Security scanning and malware removal",
        TweakCategory::System,
        RegistryTweak {
            id: TweakId::DisableWindowsMaintenance,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Schedule\\Maintenance",
                        key: "MaintenanceDisabled",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Schedule\\Maintenance",
                        key: "MaintenanceDisabled",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        true, // Requires reboot
    )
}

pub fn disable_speculative_execution_mitigations<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Disable Speculation Mitigations",
        "Controls CPU security mitigations for speculative execution vulnerabilities (Spectre, Meltdown, etc.). Settings:\n\
        • Default (0): All mitigations enabled as recommended by Microsoft\n\
        • Disabled (3): Disables all software-based mitigations",
        TweakCategory::Security,
        RegistryTweak {
            id: TweakId::SpeculativeExecutionMitigations,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                            key: "FeatureSettingsOverride",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                            key: "FeatureSettingsOverrideMask",
                            value: RegistryKeyValue::Dword(3),
                        },
                    ],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                            key: "FeatureSettingsOverride",
                            value: RegistryKeyValue::Dword(3),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                            key: "FeatureSettingsOverrideMask",
                            value: RegistryKeyValue::Dword(3),
                        },
                    ],
                ),
            ]),
        },
        true, // Correctly requires reboot
    )
}

pub fn high_performance_visual_settings<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Visual Performance Settings",
        "Configures Windows visual effects to optimize for performance or visual quality. Affects:\n\
        Animation Effects:\n\
        • Window minimize/maximize animations\n\
        • Menu and tooltip fading\n\
        • Smooth-scrolling for lists\n\
        • Task switching animations\n\
        • Animation effects in windows and taskbar\n\
        • Animate controls and elements inside windows\n\
        Visual Effects:\n\
        • Desktop composition (transparency)\n\
        • Window shadow effects\n\
        • Taskbar thumbnail previews\n\
        • Aero Peek desktop preview\n\
        • Mouse pointer shadows\n\
        • Font smoothing (ClearType)\n\
        • Thumbnail cache for Windows Explorer\n\
        Performance Impact:\n\
        • Most significant on systems with integrated graphics\n\
        • Minimal impact on systems with dedicated GPUs\n\
        • Can improve responsiveness on low-end systems\n\
        • May reduce GPU memory usage",
        TweakCategory::Graphics,
        RegistryTweak {
            id: TweakId::HighPerformanceVisualSettings,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![
                        // Windows default visual settings
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects",
                            key: "VisualFXSetting",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop\\WindowMetrics",
                            key: "MinAnimate",
                            value: RegistryKeyValue::Dword(1),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "UserPreferencesMask",
                            value: RegistryKeyValue::Binary(vec![158, 30, 7, 128, 18, 0, 0, 0]),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\DWM",
                            key: "EnableAeroPeek",
                            value: RegistryKeyValue::Dword(1),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                            key: "ExtendedUIHoverTime",
                            value: RegistryKeyValue::Dword(400),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "SmoothScroll",
                            value: RegistryKeyValue::Dword(1),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "FontSmoothing",
                            value: RegistryKeyValue::Dword(2),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Cursors",
                            key: "CursorShadow",
                            value: RegistryKeyValue::Dword(1),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                            key: "EnableTransparentGlass",
                            value: RegistryKeyValue::Dword(1),
                        },
                        // Additional settings for default
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                            key: "TaskbarAnimations",
                            value: RegistryKeyValue::Dword(1),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\DWM",
                            key: "AlwaysHibernateThumbnails",
                            value: RegistryKeyValue::Dword(1),
                        },
                    ],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![
                        // Disable all visual effects
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects",
                            key: "VisualFXSetting",
                            value: RegistryKeyValue::Dword(2),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop\\WindowMetrics",
                            key: "MinAnimate",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "UserPreferencesMask",
                            value: RegistryKeyValue::Binary(vec![144, 18, 3, 128, 16, 0, 0, 0]),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\DWM",
                            key: "EnableAeroPeek",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                            key: "ExtendedUIHoverTime",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "SmoothScroll",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "FontSmoothing",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Cursors",
                            key: "CursorShadow",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                            key: "EnableTransparentGlass",
                            value: RegistryKeyValue::Dword(0),
                        },
                        // Additional performance settings
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                            key: "TaskbarAnimations",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\DWM",
                            key: "AlwaysHibernateThumbnails",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                            key: "ListviewAlphaSelect",
                            value: RegistryKeyValue::Dword(0),
                        },
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                            key: "ListviewShadow",
                            value: RegistryKeyValue::Dword(0),
                        },
                    ],
                ),
            ]),
        },
        false, // Changes take effect on explorer restart
    )
}

pub fn split_large_caches<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Cache Splitting",
        "Controls how Windows handles large memory caches in kernel mode. When enabled:\n\
        • Splits large kernel caches into smaller segments\n\
        • Can improve performance on systems with >32GB RAM\n\
        • Reduces cache contention in multi-core systems\n\
        • May improve memory allocation efficiency\n\
        Best used on high-performance systems running memory-intensive workloads. Not recommended for systems with less than 16GB RAM as it can increase memory overhead.",
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::SplitLargeCaches,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                        key: "SplitLargeCaches",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                        key: "SplitLargeCaches",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        true, // Correctly requires reboot
    )
}

pub fn disable_protected_services<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Protected Network Services",
        "Controls core Windows networking services that are normally protected from being disabled. Affected services:\n\
        • DoSvc (Delivery Optimization): P2P Windows Updates\n\
        • DHCP: Network address assignment\n\
        • NCB: Network Connection Broker\n\
        • Netprofm: Network List Service\n\
        • NSI: Network Store Interface\n\
        • RmSvc: Radio Management\n\n\
        WARNING: Disabling these services will break:\n\
        • Network connectivity (both wired and wireless)\n\
        • Windows Update functionality\n\
        • Network discovery features",
        TweakCategory::Services,
        RegistryTweak {
            id: TweakId::DisableProtectedServices,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\DoSvc",
                            key: "Start",
                            value: RegistryKeyValue::Dword(3),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\Dhcp",
                            key: "Start",
                            value: RegistryKeyValue::Dword(2),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\NcbService",
                            key: "Start",
                            value: RegistryKeyValue::Dword(2),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\netprofm",
                            key: "Start",
                            value: RegistryKeyValue::Dword(2),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\nsi",
                            key: "Start",
                            value: RegistryKeyValue::Dword(2),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\RmSvc",
                            key: "Start",
                            value: RegistryKeyValue::Dword(2),
                        },
                    ],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\DoSvc",
                            key: "Start",
                            value: RegistryKeyValue::Dword(4),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\Dhcp",
                            key: "Start",
                            value: RegistryKeyValue::Dword(4),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\NcbService",
                            key: "Start",
                            value: RegistryKeyValue::Dword(4),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\netprofm",
                            key: "Start",
                            value: RegistryKeyValue::Dword(4),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\nsi",
                            key: "Start",
                            value: RegistryKeyValue::Dword(4),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\RmSvc",
                            key: "Start",
                            value: RegistryKeyValue::Dword(4),
                        },
                    ],
                ),
            ]),
        },
        true, // Correctly requires reboot
    )
}

pub fn disable_security_accounts_manager<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Security Accounts Manager",
        "Controls the Security Accounts Manager (SAM) service responsible for user account management.",
        TweakCategory::Services,
        RegistryTweak {
            id: TweakId::DisableSecurityAccountsManager,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\SamSs",
                        key: "Start",
                        value: RegistryKeyValue::Dword(2),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\SamSs",
                        key: "Start",
                        value: RegistryKeyValue::Dword(4),
                    }],
                ),
            ]),
        },
        true, // Correctly requires reboot
    )
}

pub fn disable_paging_combining<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Memory Page Combining",
        "Controls Windows Memory Compression and Page Combining features that optimize RAM usage. When disabled:\n\
        • Prevents Windows from combining duplicate memory pages\n\
        • Disables memory compression for less-used pages\n\
        • May reduce CPU usage from compression operations\n\
        • Increases actual RAM usage\n\
        Best used on systems with abundant RAM (32GB+) where memory compression overhead is undesirable. Not recommended for systems with limited RAM as it can increase page file usage.",
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::DisablePagingCombining,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(false),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                        key: "DisablePagingCombining",
                        value: RegistryKeyValue::Dword(0),
                    }],
                ),
                (
                    TweakOption::Enabled(true),
                    vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                        key: "DisablePagingCombining",
                        value: RegistryKeyValue::Dword(1),
                    }],
                ),
            ]),
        },
        true, // Correctly requires reboot
    )
}

pub fn enable_mcsss<'a>() -> Tweak<'a> {
    Tweak::registry_tweak(
        "Multimedia Class Scheduler Service",
        "Controls the Multimedia Class Scheduler Service (MMCSS) that manages CPU priority for multimedia applications. This service:\n\
        • Provides prioritized CPU scheduling for audio and video applications\n\
        • Reduces audio glitches and video stuttering\n\
        • Helps maintain consistent multimedia performance\n\
        Available modes:\n\
        • Auto (3): Service starts only when needed\n\
        • Enabled (2): Service runs constantly, reducing startup latency\n\
        • Disabled (4): No multimedia prioritization\n\
        Most beneficial for:\n\
        • Audio production workstations\n\
        • Video editing systems\n\
        • Gaming systems requiring consistent audio\n\
        • Real-time media streaming\n\
        Note: Enabling permanently may slightly increase CPU usage but provides more consistent multimedia performance.",
        TweakCategory::Services,
        RegistryTweak {
            id: TweakId::EnableMcsss,
            options: IndexMap::from_iter(vec![
                (
                    TweakOption::Enabled(true),
                    vec![
                        // Force services to always run
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\Audiosrv",
                            key: "Start",
                            value: RegistryKeyValue::Dword(2),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\MMCSS",
                            key: "Start",
                            value: RegistryKeyValue::Dword(2),
                        },
                        // Optional: Optimize MMCSS settings for better responsiveness
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
                            key: "NetworkThrottlingIndex",
                            value: RegistryKeyValue::Dword(0xFFFFFFFF),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
                            key: "SystemResponsiveness",
                            value: RegistryKeyValue::Dword(0),
                        },
                    ],
                ),
                (
                    TweakOption::Enabled(false),
                    vec![
                        // Completely disable multimedia prioritization
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\Audiosrv",
                            key: "Start",
                            value: RegistryKeyValue::Dword(4),
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\MMCSS",
                            key: "Start",
                            value: RegistryKeyValue::Dword(4),
                        },
                    ],
                ),
            ]),
        },
        true, // Correctly requires reboot
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Test that the tweak options are configured correctly
    ///
    /// 1. If a tweak uses "Enabled(false)", then it should only have 1 other option: "Enabled(true)"
    /// 2. If a tweak uses Option("Some Option"), then it should not have any other options with the same name, or "Enabled(true)"/"Enabled(false)"
    fn test_tweak_options_configured_correctly() {
        let tweaks = vec![
            configure_prefetcher_service(),
            disable_application_telemetry(),
            thread_dpc_disable(),
            svc_host_split_threshold(),
            disable_windows_defender(),
            disable_page_file_encryption(),
            disable_intel_tsx(),
            disable_windows_maintenance(),
            additional_kernel_worker_threads(),
            disable_speculative_execution_mitigations(),
            high_performance_visual_settings(),
            split_large_caches(),
            disable_protected_services(),
            disable_security_accounts_manager(),
            disable_paging_combining(),
            enable_mcsss(),
        ];

        for tweak in tweaks {
            let options = tweak.options;
            if options.contains(&TweakOption::Enabled(false)) {
                assert_eq!(options.len(), 2, "Tweak: {:?}", tweak.name);
                assert!(
                    options.contains(&TweakOption::Enabled(true)),
                    "Tweak: {:?}",
                    tweak.name
                );
            } else if options.contains(&TweakOption::Enabled(true)) {
                assert_eq!(options.len(), 2, "Tweak: {:?}", tweak.name);
                assert!(options.contains(&TweakOption::Enabled(false)));
            } else {
                for option in options.clone() {
                    if let TweakOption::Option(name) = option {
                        for other_option in options.clone() {
                            if let TweakOption::Option(other_name) = other_option {
                                if name == other_name {
                                    assert_eq!(name, other_name);
                                }
                            }
                        }
                        assert!(
                            !options.contains(&TweakOption::Enabled(true)),
                            "Tweak: {:?}",
                            tweak.name
                        );
                        assert!(
                            !options.contains(&TweakOption::Enabled(false)),
                            "Tweak: {:?}",
                            tweak.name
                        );
                    }
                }
            }
        }
    }
}
