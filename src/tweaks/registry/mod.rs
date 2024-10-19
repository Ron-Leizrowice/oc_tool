//  src/tweaks/registry/mod.rs

use method::{RegistryModification, RegistryTweak};

use crate::{tweaks::TweakId, utils::registry::RegistryKeyValue};

use super::{Tweak, TweakCategory};

pub mod method;

pub fn all_registry_tweaks<'a>() -> Vec<(TweakId, Tweak<'a>)> {
    vec![
        (TweakId::LargeSystemCache, enable_large_system_cache()),
        (TweakId::SystemResponsiveness, system_responsiveness()),
        (TweakId::DisableHWAcceleration, disable_hw_acceleration()),
        (TweakId::Win32PrioritySeparation, win32_priority_separation()),
        (TweakId::DisableCoreParking, disable_core_parking()),
        (TweakId::NoLowDiskSpaceChecks, disable_low_disk_space_checks()),
        (TweakId::DisableWindowsErrorReporting, disable_windows_error_reporting()),
        (TweakId::DontVerifyRandomDrivers, dont_verify_random_drivers()),
        (TweakId::DisableDriverPaging, disable_driver_paging()),
        (TweakId::DisablePrefetcher, disable_prefetcher()),
        (TweakId::DisableApplicationTelemetry, disable_application_telemetry()),
        (TweakId::ThreadDpcDisable, thread_dpc_disable()),
        (TweakId::SvcHostSplitThreshold, svc_host_split_threshold()),
        (TweakId::DisableWindowsDefender, disable_windows_defender()),
        (TweakId::DisablePageFileEncryption, disable_page_file_encryption()),
        (TweakId::DisableIntelTSX, disable_intel_tsx()),
        (TweakId::DisableWindowsMaintenance, disable_windows_maintenance()),
        (TweakId::AdditionalKernelWorkerThreads, additional_kernel_worker_threads()),
        (TweakId::DisableSpeculativeExecutionMitigations, disable_speculative_execution_mitigations()),
        (TweakId::HighPerformanceVisualSettings, high_performance_visual_settings()),
        (TweakId::EnhancedKernelPerformance, enhanced_kernel_performance()),
        (TweakId::SplitLargeCaches, split_large_caches()),
        (TweakId::DisableProtectedServices, disable_protected_services()),
        (TweakId::DisablePagingCombining, disable_paging_combining()),
        (TweakId::DisableSecurityAccountsManager, disable_security_accounts_manager()),
        (TweakId::AggressiveDpcHandling, aggressive_dpc_handling()),
        (TweakId::EnableMcsss, enable_mcsss()),

 ]
}

        pub fn enable_large_system_cache<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Large System Cache",
                "Optimizes system memory management by adjusting the LargeSystemCache setting.",
                TweakCategory::Memory,
                RegistryTweak {
                    id: TweakId::LargeSystemCache,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                            key: "LargeSystemCache",
                            // Windows will act as a server, optimizing for file sharing and network operations, potentially improving RAM disk performance.
                            target_value: RegistryKeyValue::Dword(1),
                            // Windows will favor foreground applications in terms of memory allocation.
                            default_value: Some(RegistryKeyValue::Dword(0)),
                        },
                    ],
                },
                true, // requires reboot
            )
        }
        
        pub fn system_responsiveness<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "System Responsiveness",
                "Optimizes system responsiveness by adjusting the SystemResponsiveness setting.",
                TweakCategory::System,
                RegistryTweak {
                    id: TweakId::SystemResponsiveness,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile",
                            key: "SystemResponsiveness",
                            // Windows will favor foreground applications in terms of resource allocation.
                            target_value: RegistryKeyValue::Dword(0),
                            // Windows will favor background services in terms of resource allocation.
                            default_value: Some(RegistryKeyValue::Dword(20)),
        
                        },
                    ],
                },
                false, // does not require reboot
            )
        }
        
        pub fn disable_hw_acceleration<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Hardware Acceleration",
                "Disables hardware acceleration for the current user.",
                TweakCategory::Graphics,
                RegistryTweak {
                    id: TweakId::DisableHWAcceleration,
                    modifications: vec![RegistryModification {
                        path: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics",
                        key: "DisableHWAcceleration",
                        target_value: RegistryKeyValue::Dword(1),
                        default_value: Some(RegistryKeyValue::Dword(0)),
                    }],
                },
                true, // requires reboot
            )
        }
        
        pub fn win32_priority_separation<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Win32PrioritySeparation",
                "Optimizes system responsiveness by adjusting the Win32PrioritySeparation setting.",
                TweakCategory::System,
                RegistryTweak {
                    id: TweakId::Win32PrioritySeparation,
                    modifications: vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\PriorityControl",
                        key: "Win32PrioritySeparation",
                        // Foreground applications will receive priority over background services.
                        target_value: RegistryKeyValue::Dword(26),
                        // Background services will receive priority over foreground applications.
                        default_value: Some(RegistryKeyValue::Dword(2)),
                    }],
                },
                false, // does not require reboot
            )
        }
        
        pub fn disable_core_parking<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Core Parking",
                "Disables core parking to improve system performance.",
                TweakCategory::Power,
                RegistryTweak {
                    id: TweakId::DisableCoreParking,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583",
                            key: "ValueMax",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(64)),
                        },
                    ],
                },
                true, // requires reboot
            )
        }
        
        pub fn disable_low_disk_space_checks<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Low Disk Space Checks",
                "Disables low disk space checks to prevent notifications.",
                TweakCategory::System,
                RegistryTweak {
                    id: TweakId::NoLowDiskSpaceChecks,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer",
                            key: "NoLowDiskSpaceChecks",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: Some(RegistryKeyValue::Dword(0)),
                        },
                    ],
                },
                true, // requires reboot
            )
        }
        
        pub fn disable_windows_error_reporting<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Windows Error Reporting",
                "Disables Windows Error Reporting by setting the `Disabled` registry value to `1`. This prevents the system from sending error reports to Microsoft but may hinder troubleshooting.",
                TweakCategory::Telemetry,
                RegistryTweak {
                    id: TweakId::DisableWindowsErrorReporting,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\Windows Error Reporting",
                            key: "Disabled",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: Some(RegistryKeyValue::Dword(0)),
                        },
                    ],
                },
                false,
            )
        }
        
        pub fn dont_verify_random_drivers<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Random Driver Verification",
                "Disables random driver verification to improve system performance.",
                TweakCategory::System,
                RegistryTweak {
                    id: TweakId::DontVerifyRandomDrivers,
                    modifications: vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem",
                        key: "DontVerifyRandomDrivers",
                        target_value: RegistryKeyValue::Dword(1),
                        default_value: None,
                    }],
                },
                false,
            )
        }
        
        pub fn disable_driver_paging<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Driver Paging",
                " Paging executive is used to load system files such as kernel and hardware drivers to the page file when needed. Disable will force run into not virtual memory",
                TweakCategory::Memory,
                RegistryTweak {
                    id: TweakId::DisableDriverPaging,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Session Manager\\Memory Management",
                            key: "DisablePagingExecutive",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                    ],
                },
                false,
            )
        }
        
        pub fn disable_prefetcher<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Prefetcher Service",
                "Disables the Prefetcher service by setting the `EnablePrefetcher` registry value to `0`. This may reduce system boot time and improve performance, especially on systems with SSDs.",
                TweakCategory::Services,
                RegistryTweak {
                    id: TweakId::DisablePrefetcher,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\\PrefetchParameters",
                            key: "EnablePrefetcher",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(3)),
                        },
                    ],
                },
                false,
            )
        }
        
        pub fn disable_application_telemetry<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Application Telemetry",
                "Disables Windows Application Telemetry by setting the `AITEnable` registry value to `0`. This reduces the collection of application telemetry data but may limit certain features or diagnostics.",
                TweakCategory::Telemetry,
                RegistryTweak {
                    id: TweakId::DisableApplicationTelemetry,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows\\AppCompat",
                            key: "AITEnable",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: None,
                        },
                    ],
                },
                false,
            )
        }
        
        pub fn thread_dpc_disable<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Thread DPC Disable",
                "Disables or modifies the handling of Deferred Procedure Calls (DPCs) related to threads by setting the 'ThreadDpcEnable' registry value to 0. This aims to reduce DPC overhead and potentially enhance system responsiveness. However, it may lead to system instability or compatibility issues with certain hardware or drivers.",
                TweakCategory::Kernel,
                RegistryTweak {
                    id: TweakId::ThreadDpcDisable,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "ThreadDpcEnable",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: None,
                        },
                    ],
                },
                false,
            )
        }
        
        pub fn svc_host_split_threshold<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable SvcHost Split",
                "Adjusts the SvcHost Split Threshold in KB to optimize system performance.",
                TweakCategory::System,
                RegistryTweak {
                    id: TweakId::SvcHostSplitThreshold,
                    modifications: vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control",
                        key: "SvcHostSplitThresholdInKB",
                        target_value: RegistryKeyValue::Dword(0x0f000000),
                        default_value: None,
                    }],
                },
                true,
            )
        }
        
        pub fn disable_windows_defender<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Windows Defender",
                "Disables Windows Defender by setting the `DisableAntiSpyware` registry value to `1`. This prevents Windows Defender from running and may leave your system vulnerable to malware.",
                TweakCategory::Security,
                RegistryTweak {
                    id: TweakId::DisableWindowsDefender,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Policies\\Microsoft\\Windows Defender",
                            key: "DisableAntiSpyware",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                    ],
                },
                false,
            )
        }
        
        pub fn disable_page_file_encryption<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Page File Encryption",
                "Disables page file encryption to improve system performance.",
                TweakCategory::Memory,
                RegistryTweak {
                    id: TweakId::DisablePageFileEncryption,
                    modifications: vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\FileSystem",
                        key: "NtfsEncryptPagingFile",
                        target_value: RegistryKeyValue::Dword(0),
                        default_value: Some(RegistryKeyValue::Dword(1)),
                    }],
                },
                true,
            )
        }
        
        pub fn disable_intel_tsx<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Intel TSX",
                "Disables Intel Transactional Synchronization Extensions (TSX) operations to mitigate potential security vulnerabilities.",
                TweakCategory::Security,
                RegistryTweak {
                    id: TweakId::DisableIntelTSX,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Kernel",
                            key: "DisableTsx",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                    ],
                },
                true,
            )
        }
        
        pub fn disable_windows_maintenance<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Windows Maintenance",
                "Disables Windows Maintenance by setting the `MaintenanceDisabled` registry value to `1`. This prevents Windows from performing maintenance tasks, such as software updates, system diagnostics, and security scans.",
                TweakCategory::Action,
                RegistryTweak {
                    id: TweakId::DisableWindowsMaintenance,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Schedule\\Maintenance",
                            key: "MaintenanceDisabled",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                    ],
                },
                false, // requires reboot
            )
        }
        
        pub fn additional_kernel_worker_threads<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Additional Worker Threads",
                "Increases the number of kernel worker threads by setting the AdditionalCriticalWorkerThreads and AdditionalDelayedWorkerThreads values to match the number of logical processors in the system. This tweak boosts performance in multi-threaded workloads by allowing the kernel to handle more concurrent operations, improving responsiveness and reducing bottlenecks in I/O-heavy or CPU-bound tasks. It ensures that both critical and delayed work items are processed more efficiently, particularly on systems with multiple cores.",
                TweakCategory::Kernel,
                RegistryTweak {
                    id: TweakId::AdditionalKernelWorkerThreads,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive",
                            key: "AdditionalCriticalWorkerThreads",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive",
                            key: "AdditionalDelayedWorkerThreads",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: None,
                        },
                    ],
                },
                false,
            )
        }
        
        pub fn disable_speculative_execution_mitigations<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Speculative Execution Mitigations",
                "
        Disables speculative execution mitigations by setting the `FeatureSettingsOverride` and `FeatureSettingsOverrideMask` registry values to `3`. This may improve performance but can also introduce security risks.
        ".trim(),
                TweakCategory::Security,
                RegistryTweak {
                    id: TweakId::DisableSpeculativeExecutionMitigations,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                            key: "FeatureSettingsOverride",
                            target_value: RegistryKeyValue::Dword(3),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                            key: "FeatureSettingsOverrideMask",
                            target_value: RegistryKeyValue::Dword(3),
                            default_value: None,
                        },
                    ],
                },
                true,
            )
        }
        
        pub fn high_performance_visual_settings<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "High Performance Visual Settings",
                "
        This tweak adjusts Windows visual settings to prioritize performance over appearance:
        
        1. Sets the overall Visual Effects setting to 'Adjust for best performance'
        2. Disables animations when minimizing and maximizing windows
        3. Turns off animated controls and elements inside windows
        4. Disables Aero Peek (the feature that shows desktop previews when hovering over the Show Desktop button)
        5. Turns off live thumbnails for taskbar previews
        6. Disables smooth scrolling of list views
        7. Turns off fading effects for menus and tooltips
        8. Disables font smoothing (ClearType)
        9. Turns off the shadow effect under mouse pointer
        10. Disables the shadow effect for window borders
        ".trim(),
                TweakCategory::Graphics,
                RegistryTweak {
                    id: TweakId::HighPerformanceVisualSettings,
                    modifications: vec![
                        // 1. Set VisualFXSetting to 'Adjust for best performance' (2)
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects",
                            key: "VisualFXSetting",
                            target_value: RegistryKeyValue::Dword(2),
                            default_value: Some(RegistryKeyValue::Dword(0)), // Default VisualFXSetting
                        },
                        // 2. Disable animations when minimizing/maximizing windows
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop\\WindowMetrics",
                            key: "MinAnimate",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(1)),
                        },
                        // 3. Turn off animated controls and elements inside windows
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "UserPreferencesMask",
                            target_value: RegistryKeyValue::Binary(vec![144, 18, 3, 128, 16, 0, 0, 0]),
                            default_value: Some(RegistryKeyValue::Binary(vec![158, 30, 7, 128, 18, 0, 0, 0])),
                        },
                        // 4. Disable Aero Peek
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\DWM",
                            key: "EnableAeroPeek",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(1)),
                        },
                        // 5. Turn off live thumbnails for taskbar previews
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                            key: "ExtendedUIHoverTime",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(400)), // Default hover time
                        },
                        // 6. Disable smooth scrolling of list views
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "SmoothScroll",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(1)),
                        },
                        // 7. Turn off fading effects for menus and tooltips
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "UserPreferencesMask",
                            target_value: RegistryKeyValue::Binary(vec![144, 18, 3, 128, 16, 0, 0, 0]),
                            default_value: Some(RegistryKeyValue::Binary(vec![158, 30, 7, 128, 18, 0, 0, 0])),
                        },
                        // 8. Disable font smoothing (ClearType)
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Desktop",
                            key: "FontSmoothing",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(2)),
                        },
                        // 9. Turn off the shadow effect under mouse pointer
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Control Panel\\Cursors",
                            key: "CursorShadow",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(1)),
                        },
                        // 10. Disable the shadow effect for window borders
                        RegistryModification {
                            path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                            key: "EnableTransparentGlass",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: Some(RegistryKeyValue::Dword(1)),
                        },
                    ],
                },
                false, // requires reboot
            )
        }
        
        pub fn enhanced_kernel_performance<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Enhanced Kernel Performance",
                "Optimizes various kernel-level settings in the Windows Registry to improve system performance by increasing I/O queue sizes, buffer sizes, and stack sizes, while disabling certain security features. These changes aim to enhance multitasking and I/O operations but may affect system stability and security.",
                TweakCategory::Kernel,
                RegistryTweak {
                    id: TweakId::EnhancedKernelPerformance,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "MaxDynamicTickDuration",
                            target_value: RegistryKeyValue::Dword(10),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "MaximumSharedReadyQueueSize",
                            target_value: RegistryKeyValue::Dword(128),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "BufferSize",
                            target_value: RegistryKeyValue::Dword(32),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoQueueWorkItem",
                            target_value: RegistryKeyValue::Dword(32),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoQueueWorkItemToNode",
                            target_value: RegistryKeyValue::Dword(32),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoQueueWorkItemEx",
                            target_value: RegistryKeyValue::Dword(32),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoQueueThreadIrp",
                            target_value: RegistryKeyValue::Dword(32),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "ExTryQueueWorkItem",
                            target_value: RegistryKeyValue::Dword(32),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "ExQueueWorkItem",
                            target_value: RegistryKeyValue::Dword(32),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IoEnqueueIrp",
                            target_value: RegistryKeyValue::Dword(32),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "XMMIZeroingEnable",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "UseNormalStack",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "UseNewEaBuffering",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "StackSubSystemStackSize",
                            target_value: RegistryKeyValue::Dword(65536),
                            default_value: None,
                        },
                    ],
                },
                false, // does not require reboot
            )
        }
        
        pub fn split_large_caches<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Split Large Caches",
                "This registry key is used to enable the splitting of large caches in the Windows operating system. This setting can help improve system performance by optimizing how the kernel handles large cache sizes, particularly in systems with significant memory resources.",
                TweakCategory::Memory,
                RegistryTweak {
                    id: TweakId::SplitLargeCaches,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "SplitLargeCaches",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: Some(RegistryKeyValue::Dword(0)),
                        },
                    ],
                },
                true, // requires reboot
            )
        }
        
        pub fn disable_protected_services<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Protected Services",
                "Disables multiple services which can only be stopped by modifying the registry. These will not break your system, but will stop networking functionality.",
                TweakCategory::Services,
                RegistryTweak {
                    id: TweakId::DisableProtectedServices,
                    modifications: vec![
                        RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\DoSvc",
                        key: "Start",
                        target_value: RegistryKeyValue::Dword(4),
                        default_value: Some(RegistryKeyValue::Dword(3)),
                    },
                    RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\Dhcp",
                        key: "Start",
                        target_value: RegistryKeyValue::Dword(4),
                        default_value: Some(RegistryKeyValue::Dword(2)),
                    },
                    RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\NcbService",
                        key: "Start",
                        target_value: RegistryKeyValue::Dword(4),
                        default_value: Some(RegistryKeyValue::Dword(2)),
                    },
                    RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\netprofm",
                        key: "Start",
                        target_value: RegistryKeyValue::Dword(4),
                        default_value: Some(RegistryKeyValue::Dword(2)),
                    },
                    RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\nsi",
                        key: "Start",
                        target_value: RegistryKeyValue::Dword(4),
                        default_value: Some(RegistryKeyValue::Dword(2)),
                    },
                    RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\RmSvc",
                        key: "Start",
                        target_value: RegistryKeyValue::Dword(4),
                        default_value: Some(RegistryKeyValue::Dword(2)),
                    }
                    ],
                },
                true, // requires reboot
            )
        }
        
        pub fn disable_security_accounts_manager<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Security Accounts Manager",
                "Disables the Security Accounts Manager service by setting the Start registry DWORD to 4.",
                TweakCategory::Services,
                RegistryTweak {
                    id: TweakId::DisableSecurityAccountsManager,
                    modifications: vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\SamSs",
                        key: "Start",
                        target_value: RegistryKeyValue::Dword(4),
                        default_value: Some(RegistryKeyValue::Dword(2)),
                    }],
                },
                true, // requires reboot
            )
        }
        
        pub fn disable_paging_combining<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Disable Paging Combining",
                "Disables Windows attempt to save as much RAM as possible, such as sharing pages for images, copy-on-write for data pages, and compression.",
                TweakCategory::Memory,
                RegistryTweak {
                    id: TweakId::DisablePagingCombining,
                    modifications: vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management",
                        key: "DisablePagingCombining",
                        target_value: RegistryKeyValue::Dword(1),
                        default_value: None,
                    }],
                },
                true, // requires reboot
            )
        }
        
        pub fn aggressive_dpc_handling<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Aggressive DPC Handling",
                "This tweak modifies kernel-level settings in the Windows Registry to aggressively optimize the handling of Deferred Procedure Calls (DPCs) by disabling timeouts, watchdogs, and minimizing queue depth, aiming to enhance system responsiveness and reduce latency. However, it also removes safeguards that monitor and control long-running DPCs, which could lead to system instability or crashes in certain scenarios, particularly during high-performance or overclocking operations.",
                TweakCategory::Kernel,
                RegistryTweak {
                    id: TweakId::AggressiveDpcHandling,
                    modifications: vec![
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "DpcWatchdogProfileOffset",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "DpcTimeout",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "IdealDpcRate",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "MaximumDpcQueueDepth",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "MinimumDpcRate",
                            target_value: RegistryKeyValue::Dword(1),
                            default_value: None,
                        },
                        RegistryModification {
                            path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel",
                            key: "DpcWatchdogPeriod",
                            target_value: RegistryKeyValue::Dword(0),
                            default_value: None,
                        },
                    ],
                },
                false, // does not require reboot
            )
        }
        
        pub fn enable_mcsss<'a>() -> Tweak<'a> {
            Tweak::registry_tweak(
                "Enable Multimedia Class Scheduler Service",
                "Enables the Multimedia Class Scheduler Service (MMCSS) by setting the Start registry DWORD to 2.",
                TweakCategory::Services,
                RegistryTweak {
                    id: TweakId::EnableMcsss,
                    modifications: vec![RegistryModification {
                        path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\Audiosrv",
                        key: "Start",
                        target_value: RegistryKeyValue::Dword(2),
                        default_value: Some(RegistryKeyValue::Dword(3)),
                    }],
                },
                true, // requires reboot
            )
        }
        