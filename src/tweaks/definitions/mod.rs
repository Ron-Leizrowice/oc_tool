mod disable_processor_idle_states;
mod kill_explorer;
mod kill_non_critical_services;
mod low_res_mode;
mod slow_mode;
mod ultimate_performance_plan;
use std::collections::BTreeMap;

use disable_processor_idle_states::DisableProcessIdleStates;
use kill_explorer::KillExplorerTweak;
use kill_non_critical_services::KillNonCriticalServicesTweak;
use low_res_mode::LowResMode;
use slow_mode::SlowMode;
use ultimate_performance_plan::UltimatePerformancePlan;

use super::{
    group_policy::{GroupPolicyTweak, GroupPolicyValue},
    msr::MSRTweak,
    powershell::PowershellTweak,
    registry::{RegistryKeyValue, RegistryModification, RegistryTweak},
    Tweak, TweakCategory,
};
use crate::widgets::TweakWidget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum TweakId {
    LargeSystemCache,
    SystemResponsiveness,
    DisableHWAcceleration,
    Win32PrioritySeparation,
    DisableCoreParking,
    ProcessIdleTasks,
    SeLockMemoryPrivilege,
    UltimatePerformancePlan,
    NoLowDiskSpaceChecks,
    AdditionalKernelWorkerThreads,
    DisableHPET,
    AggressiveDpcHandling,
    EnhancedKernelPerformance,
    DisableRamCompression,
    DisableApplicationTelemetry,
    DisableWindowsErrorReporting,
    DisableLocalFirewall,
    DontVerifyRandomDrivers,
    DisableDriverPaging,
    DisablePrefetcher,
    DisableSuccessAuditing,
    ThreadDpcDisable,
    SvcHostSplitThreshold,
    DisablePagefile,
    DisableSpeculativeExecutionMitigations,
    DisableDataExecutionPrevention,
    DisableWindowsDefender,
    DisablePageFileEncryption,
    DisableProcessIdleStates,
    KillAllNonCriticalServices,
    DisableIntelTSX,
    DisableWindowsMaintenance,
    KillExplorer,
    HighPerformanceVisualSettings,
    LowResMode,
    SplitLargeCaches,
    DisableProtectedServices,
    DisableSecurityAccountsManager,
    DisablePagingCombining,
    DisableSuperfetch,
    SlowMode,
    EnableMcsss,
    DisbleCpb,
    SpeculativeStoreBypassDisable,
    PredictiveStoreForwardingDisable,
}

/// Initializes all tweaks with their respective configurations.
pub fn all<'a>() -> BTreeMap<TweakId, Tweak<'a>> {
    BTreeMap::from_iter(vec![
        (TweakId::ProcessIdleTasks, process_idle_tasks()),
        (TweakId::LowResMode, low_res_mode()),
        (TweakId::LargeSystemCache, enable_large_system_cache()),
        (TweakId::SystemResponsiveness, system_responsiveness()),
        (TweakId::DisableHWAcceleration, disable_hw_acceleration()),
        (
            TweakId::Win32PrioritySeparation,
            win32_priority_separation(),
        ),
        (TweakId::DisableCoreParking, disable_core_parking()),
        (TweakId::SeLockMemoryPrivilege, se_lock_memory_privilege()),
        (
            TweakId::UltimatePerformancePlan,
            ultimate_performance_plan(),
        ),
        (
            TweakId::NoLowDiskSpaceChecks,
            disable_low_disk_space_checks(),
        ),
        (
            TweakId::AdditionalKernelWorkerThreads,
            additional_kernel_worker_threads(),
        ),
        (TweakId::DisableHPET, disable_hpet()),
        (TweakId::AggressiveDpcHandling, aggressive_dpc_handling()),
        (
            TweakId::EnhancedKernelPerformance,
            enhanced_kernel_performance(),
        ),
        (TweakId::DisableRamCompression, disable_ram_compression()),
        (
            TweakId::DisableApplicationTelemetry,
            disable_application_telemetry(),
        ),
        (
            TweakId::DisableWindowsErrorReporting,
            disable_windows_error_reporting(),
        ),
        (TweakId::DisableLocalFirewall, disable_local_firewall()),
        (
            TweakId::DontVerifyRandomDrivers,
            dont_verify_random_drivers(),
        ),
        (TweakId::DisableDriverPaging, disable_driver_paging()),
        (TweakId::DisablePrefetcher, disable_prefetcher()),
        (TweakId::DisableSuccessAuditing, disable_success_auditing()),
        (TweakId::ThreadDpcDisable, thread_dpc_disable()),
        (TweakId::SvcHostSplitThreshold, svc_host_split_threshold()),
        (TweakId::DisablePagefile, disable_pagefile()),
        (
            TweakId::DisableSpeculativeExecutionMitigations,
            disable_speculative_execution_mitigations(),
        ),
        (
            TweakId::DisableDataExecutionPrevention,
            disable_data_execution_prevention(),
        ),
        (TweakId::DisableWindowsDefender, disable_windows_defender()),
        (
            TweakId::DisablePageFileEncryption,
            disable_page_file_encryption(),
        ),
        (
            TweakId::DisableProcessIdleStates,
            disable_process_idle_states(),
        ),
        (
            TweakId::KillAllNonCriticalServices,
            kill_all_non_critical_services(),
        ),
        (TweakId::DisableIntelTSX, disable_intel_tsx()),
        (
            TweakId::DisableWindowsMaintenance,
            disable_windows_maintenance(),
        ),
        (TweakId::KillExplorer, kill_explorer()),
        (
            TweakId::HighPerformanceVisualSettings,
            high_performance_visual_settings(),
        ),
        (TweakId::SplitLargeCaches, split_large_caches()),
        (
            TweakId::DisableProtectedServices,
            disable_protected_services(),
        ),
        (
            TweakId::DisableSecurityAccountsManager,
            disable_security_accounts_manager(),
        ),
        (TweakId::DisablePagingCombining, disable_paging_combining()),
        (TweakId::DisableSuperfetch, disable_superfetch()),
        (TweakId::SlowMode, slow_mode()),
        (TweakId::EnableMcsss, enable_mcsss()),
        (TweakId::DisbleCpb, disable_cpb()),
    ])
}

pub fn disable_process_idle_states<'a>() -> Tweak<'a> {
    Tweak::rust_tweak(
        "Disable Process Idle States",
        "Disables processor idle states (C-states) to prevent the CPU from entering low-power states during idle periods. This tweak can improve system responsiveness but may increase power consumption and heat output.",
        TweakCategory::Power,
        DisableProcessIdleStates::new(),
        &TweakWidget::Toggle,
        false,
    )
}

pub fn kill_all_non_critical_services<'a>() -> Tweak<'a> {
    Tweak::rust_tweak(
        "Kill All Non-Critical Services",
        "Stops all non-critical services to free up system resources and improve performance. This tweak may cause system instability or data loss.",
        TweakCategory::Services,
        KillNonCriticalServicesTweak {
            id: TweakId::KillAllNonCriticalServices,
        },
        &TweakWidget::Button,
        false,
    )
}

/// Initializes the Kill Explorer tweak.
pub fn kill_explorer<'a>() -> Tweak<'a> {
    Tweak::rust_tweak(
        "Kill Explorer",
        "Terminates the Windows Explorer process and prevents it from automatically restarting. This can free up system resources but will remove the desktop interface. Use with caution.",
        TweakCategory::Action,
        KillExplorerTweak {
            id: TweakId::KillExplorer,
        },
        &TweakWidget::Toggle,
        false,
    )
}

pub fn disable_hpet<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable Dynamic Tick",
        "Disables the dynamic tick feature, which normally reduces timer interrupts during idle periods to conserve power. By disabling dynamic tick, the system maintains a constant rate of timer interrupts, improving performance in real-time applications by reducing latency and jitter. This tweak is useful in scenarios where consistent, low-latency processing is required, but it may increase power consumption as the CPU will not enter low-power states as frequently.",
        TweakCategory::System,
        PowershellTweak {
            id: TweakId::DisableHPET,
            read_script: Some(r#"(bcdedit /enum | Select-String "useplatformclock").ToString().Trim()"#),

            apply_script: r#"
            bcdedit /deletevalue useplatformclock
            bcdedit /set disabledynamictick yes
            "#.trim(),

            undo_script: Some(r#"
            bcdedit /set useplatformclock true
            bcdedit /set disabledynamictick no
            "#.trim()),

            target_state: Some("useplatformclock        Yes".trim()),
        },
        true,
    )
}

pub fn disable_ram_compression<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable RAM Compression",
        "Disables the RAM compression feature in Windows to potentially improve system performance by reducing CPU overhead. This may lead to higher memory usage.",
        TweakCategory::Memory,
        PowershellTweak {
            id: TweakId::DisableRamCompression,
            read_script: Some(
                r#"
(Get-MMAgent | Out-String -Stream | Select-String -Pattern "MemoryCompression").ToString().Trim() -Match "False"
                "#
                .trim(),
            ),
            apply_script:
                r#"Disable-MMAgent -mc"#
                .trim(),
            undo_script: Some(
                r#"Enable-MMAgent -mc"#
                .trim(),
            ),
            target_state: Some("True"),
        },
        true,

    )
}

pub fn disable_local_firewall<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable Local Firewall",
        "Disables the local Windows Firewall for all profiles by setting the firewall state to `off`. **Warning:** This exposes the system to potential security threats and may cause issues with IPsec server connections.",
        TweakCategory::Security,
        PowershellTweak {
            id: TweakId::DisableLocalFirewall,
            read_script: Some(
                r#"
                $firewallState = netsh advfirewall show allprofiles state | Select-String "State" | ForEach-Object { $_.Line }

                if ($firewallState -match "off") {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#
                .trim(),
            ),
            apply_script:
                r#"
                try {
                    netsh advfirewall set allprofiles state off
                    Write-Output "Disable Local Firewall Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable Local Firewall Tweaks: $_"
                }
                "#
                .trim(),
            undo_script: Some(
                r#"
                try {
                    netsh advfirewall set allprofiles state on
                    Write-Output "Disable Local Firewall Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable Local Firewall Tweaks: $_"
                }
                "#
                .trim(),
            ),
            target_state: Some("Enabled"),
        },
        true,

    )
}

pub fn disable_success_auditing<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable Success Auditing",
        "Disables auditing of successful events across all categories, reducing the volume of event logs and system overhead. Security events in the Windows Security log are not affected.",
        TweakCategory::Security,
        PowershellTweak {
            id: TweakId::DisableSuccessAuditing,
            read_script: Some(
                r#"
# Ensure $auditSettings is an array
$auditSettings = @(AuditPol /get /category:* | Where-Object { $_ -match "No Auditing" })

# Check if the array has elements using .Count
if ($auditSettings.Count -gt 0) {
    $result = "Enabled"
} else {
    $result = "Disabled"
}

# Output the result
Write-Output $result
                "#
                .trim(),
            ),
            apply_script:
                r#"
                try {
                    Auditpol /set /category:* /Success:disable
                    Write-Output "Disable Success Auditing Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable Success Auditing: $_"
                }
                "#
                .trim(),

            undo_script: Some(
                r#"
                try {
                    Auditpol /set /category:* /Success:enable
                    Write-Output "Disable Success Auditing Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable Success Auditing: $_"
                }
                "#
                .trim(),
            ),
            target_state: Some("Enabled"),
        },
        true,

    )
}

pub fn disable_pagefile<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable Pagefile",
        "Disables the Windows page file, which is used as virtual memory when physical memory is full. This tweak can improve system performance by reducing disk I/O and preventing paging, but it may cause system instability or application crashes if the system runs out of memory.",
        TweakCategory::Memory,
        PowershellTweak {
            id: TweakId::DisablePagefile,
            read_script: Some(
                r#"
                $pagefileSettings = Get-WmiObject -Class Win32_PageFileUsage | Select-Object -ExpandProperty AllocatedBaseSize

                if ($pagefileSettings -eq 0) {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#.trim(),
            ),
            apply_script:"fsutil behavior set encryptpagingfile 0",
            undo_script: Some(
               "fsutil behavior set encryptpagingfile 1",
            ),
            target_state: Some("Enabled"),
        },
        true,

    )
}

pub fn disable_data_execution_prevention<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable Data Execution Prevention",
        "Disables Data Execution Prevention (DEP) by setting the `nx` boot configuration option to `AlwaysOff`. This may improve compatibility with older applications but can introduce security risks.",
        TweakCategory::Security,
        PowershellTweak {
            id: TweakId::DisableDataExecutionPrevention,
            read_script: Some(
                r#"
                $depSettings = bcdedit /enum | Select-String 'nx'

                if ($depSettings -match 'AlwaysOff') {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#
                .trim(),
            ),
            apply_script: "bcdedit.exe /set nx AlwaysOff",
            undo_script: Some(
                "bcdedit.exe /set nx OptIn",
            ),
            target_state: Some("Enabled"),
        },
        true,
    )
}

pub fn disable_superfetch<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable Superfetch",
        "Disables the Superfetch service, which preloads frequently used applications into memory to improve performance. This tweak can reduce disk I/O and memory usage but may impact performance in some scenarios.",
        TweakCategory::Memory,
        PowershellTweak {
            id: TweakId::DisableSuperfetch,
            read_script: Some(
                r#"
                $superfetchStatus = Get-Service -Name SysMain | Select-Object -ExpandProperty Status

                if ($superfetchStatus -eq "Running") {
                    "Disabled"
                } else {
                    "Enabled"
                }
                "#
                .trim(),
            ),
            apply_script: r#"Stop-Service -Force -Name "SysMain"; Set-Service -Name "SysMain" -StartupType Disabled"#,
            undo_script: Some(
                r#"Set-Service -Name "SysMain" -StartupType Automatic -Status Running"#,
            ),
            target_state: Some("Enabled"),
        },
        true, // requires reboot
    )
}

/// Function to create the `Low Resolution Mode` Rust tweak.
pub fn low_res_mode<'a>() -> Tweak<'a> {
    let method = LowResMode::default();

    let formatted_description = format!(
            "Sets the display to lower resolution and refresh rate to reduce GPU load and improve performance -> {}x{} @{}hz.",
            method.target_state.width, method.target_state.height, method.target_state.refresh_rate
        );
    let description: &'a str = Box::leak(formatted_description.into_boxed_str());

    Tweak::rust_tweak(
        "Low Resolution Mode",
        description,
        TweakCategory::Graphics,
        method,
        &TweakWidget::Toggle,
        false,
    )
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

pub fn se_lock_memory_privilege<'a>() -> Tweak<'a> {
    Tweak::group_policy_tweak(
        "SeLockMemoryPrivilege",
        "The SeLockMemoryPrivilege group policy setting allows a process to lock pages in physical memory, preventing them from being paged out to disk. This can improve performance for applications that require fast, consistent access to critical data by keeping it always available in RAM.",
        TweakCategory::Memory,
        GroupPolicyTweak {
            id: TweakId::SeLockMemoryPrivilege,
            key: "SeLockMemoryPrivilege",
            value: GroupPolicyValue::Enabled,
        },
        true,
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

pub fn ultimate_performance_plan<'a>() -> Tweak<'a> {
    Tweak::rust_tweak(
        "Enable Ultimate Performance Plan",
        "Activates the Ultimate Performance power plan, which is tailored for demanding workloads by minimizing micro-latencies and boosting hardware performance. It disables power-saving features like core parking, hard disk sleep, and processor throttling, ensuring CPU cores run at maximum frequency. This plan also keeps I/O devices and PCIe links at full power, prioritizing performance over energy efficiency. It's designed to reduce the delays introduced by energy-saving policies, improving responsiveness in tasks that require consistent, high-throughput system resources.",
        TweakCategory::Power,
        UltimatePerformancePlan::new(),
        &TweakWidget::Toggle,
        false, // requires reboot
    )
}

pub fn slow_mode<'a>() -> Tweak<'a> {
    Tweak::rust_tweak(
        "Slow Mode",
        "Places the system in a low-power state by:
1. Switching to the Power Saver scheme
2. Limiting max cores to 2
3. Limiting CPU frequency
4. Delaying CPU performance state transitions
",
        TweakCategory::Power,
        SlowMode::new(),
        &TweakWidget::Toggle,
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


pub fn process_idle_tasks<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Process Idle Tasks",
        "Runs the Process Idle Tasks command to optimize system performance by processing idle tasks and background maintenance activities.",
        TweakCategory::System,
        PowershellTweak {
            id: TweakId::ProcessIdleTasks,
            read_script: None,
            apply_script: r#"
                $advapi32 = Add-Type -MemberDefinition @"
                    [DllImport("advapi32.dll", EntryPoint="ProcessIdleTasks")]
                    public static extern bool ProcessIdleTasks();
                "@ -Name "Advapi32" -Namespace "Win32" -PassThru

                if ($advapi32::ProcessIdleTasks()) {
                    Write-Output "Process Idle Tasks completed successfully."
                } else {
                    Write-Error "Failed to run Process Idle Tasks."
                }
            "#,
            undo_script: None,
            target_state: None,
            
        },
        false, // does not require reboot
    )
}

pub fn disable_cpb<'a>() -> Tweak<'a> {
    Tweak::msr_tweak(
        "Enable Core Performance Boost",
        "Enables AMD Core Performance Boost (CPB) to dynamically adjust CPU clock speeds based on workload demands, improving performance in multi-threaded applications and tasks. This feature allows AMD processors to operate at higher frequencies when needed, maximizing processing power and responsiveness.",
        TweakCategory::Power,
        MSRTweak {
            id: TweakId::DisbleCpb,
            index: 0xC001_0015,
            bit: 25,
        },
        false, // does not require reboot
    )
}

// pub fn speculative_store_bypass_disable<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Speculative Store Bypass Disable",
//         "Disables Speculative Store Bypass (SSBD) to mitigate potential security vulnerabilities related to speculative execution. This feature prevents the processor from speculatively executing store operations that bypass the cache, reducing the risk of unauthorized data access or side-channel attacks.",
//         TweakCategory::Security,
//         MSRTweak {
//             id: TweakId::SpeculativeStoreBypassDisable,
//             index: 0xC000_0048,
//             bit: 2,
//         },
//         false, // does not require reboot
//     )
// }

// pub fn predictive_store_forwarding_disable<'a>() -> Tweak<'a> {
//     Tweak::msr_tweak(
//         "Predictive Store Forwarding Disable",
//         "Disables Predictive Store Forwarding (PSFD) to mitigate potential security vulnerabilities related to speculative execution. This feature prevents the processor from speculatively executing store operations that bypass the cache, reducing the risk of unauthorized data access or side-channel attacks.",
//         TweakCategory::Security,
//         MSRTweak {
//             id: TweakId::PredictiveStoreForwardingDisable,
//             index: 0xC000_0048,
//             bit: 7,
//         },
//         false, // does not require reboot
//     )
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disable_success_auditing_apply() {
        let tweak = disable_success_auditing();
        let result = tweak.apply();
        assert!(result.is_ok());
    }

    #[test]
    fn test_disable_success_auditing_revert() {
        let tweak = disable_success_auditing();
        let result = tweak.revert();
        assert!(result.is_ok());
    }
}

// AMD MSR Documentation (Zen4)
//
// MSR0000_001B [APIC Base Address] (Core::X86::Msr::APIC_BAR)
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSR0000_001B
// Bits Description
// 63:52 Reserved.
// 51:12 ApicBar[51:12]: APIC base address register. Read-write. Reset: 00_000F_EE00h. Specifies the base address,
// physical address [51:12], for the APICXX register set in xAPIC mode. See 2.1.13.2.1.2 [APIC Register Space].
// 11 ApicEn: APIC enable. Read-write. Reset: 0. 0=Disable Local APIC. 1=Local APIC is enabled in xAPIC mode.
// See 2.1.13.2.1.2 [APIC Register Space].
// 10 x2ApicEn: Extended APIC enable. Read-write. Reset: 0. 0=Disable Extended Local APIC. 1=Extended Local
// APIC is enabled in x2APIC mode. Clearing this bit after it has been set requires ApicEn to be cleared as well.
// 9 Reserved.
// 8 BSC: boot strap core. Read-write,Volatile. Reset: X. 0=The core is not the boot core of the BSP. 1=The core is
// the boot core of the BSP.
// 7:0 Reserved.
//
// MSR0000_0048 [Speculative Control] (Core::X86::Msr::SPEC_CTRL)
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSR0000_0048
// Bits Description
// 63:8 Reserved.
// 7 PSFD: Predictive Store Forwarding Disable. Read-write. Reset: 0. 1=Disable predictive store forwarding.
// 6:3 Reserved.
// 2 SSBD. Read-write. Reset: 0. Speculative Store Bypass Disable.
// 1 STIBP. Read-write. Reset: 0. Single thread indirect branch predictor.
// 0 IBRS. Read-write. Reset: 0. Indirect branch restriction speculation.
//
// MSR0000_0049 [Prediction Command] (Core::X86::Msr::PRED_CMD)
// _ccd[11:0]_lthree0_core[7:0]; MSR0000_0049
// Bits Description
// 63:1 Reserved.
// 0 IBPB: indirect branch prediction barrier. Write-only,Error-on-read. Reset: 0. Supported if
// Core::X86::Cpuid::FeatureExtIdEbx[IBPB] == 1.
//
// MSR0000_017B [Global Machine Check Exception Reporting Control] (Core::X86::Msr::MCG_CTL)
// Reset: 0000_0000_0000_0000h.
// This register controls enablement of the individual error reporting banks; see 3.1 [Machine Check Architecture] and
// 3.1.2.1 [Global Registers]. When a machine check register bank is not enabled in MCG_CTL, errors for that bank are not
// logged or reported, and actions enabled through the MCA are not taken; each MCi_CTL register identifies which errors
// are still corrected when MCG_CTL[i] is disabled.
// _ccd[11:0]_lthree0_core[7:0]_thread[1:0]; MSR0000_017B
// Bits Description
// 63:7 MCnEn. Configurable. Reset: 000_0000_0000_0000h.
// Description: 1=The MC machine check register bank is enabled. Width of this field is SOC implementation and
// configuration specific.
// See 3.1.2.1 [Global Registers].
// 6:0 MCnEnCore. Read-write. Reset: 00h. 1=The MC machine check register bank is enabled.
// ValidValues:
// Bit Description
// [0] Enable MCA for LSDC
// [1] Enable MCA for ICBP
// [2] Enable MCA for L2
// [3] Enable MCA for DE
// [4] Reserved.
// [5] Enable MCA for SCEX
//
// MSRC000_0080 [Extended Feature Enable] (Core::X86::Msr::EFER)
// SKINIT Execution: 0000_0000_0000_0000h.
// _ccd[11:0]_lthree0_core[7:0]_thread[1:0]; MSRC000_0080
// Bits Description
// 63:22 Reserved.
// 21 AutomaticIBRSEn: Automatic IBRS Enable. Read-write. Reset: 0. 0=IBRS protection is not enabled unless
// (SPEC_CTRL[IBRS] == 1). 1=IBRS protection is enabled for any process running at (CPL < 3) or ((ASID == 0)
// && SEV-SNP).
// 20 UAIE: Upper Address Ignore Enable. Read-write. Reset: 0. Upper Address Ignore suppresses canonical faults
// for most data access virtual addresses, which allows software to use the upper bits of a virtual address as tags.
// 19 Reserved.
// 18 IntWbinvdEn. Read-write. Reset: 0. Interruptible wbinvd, wbnoinvd enable.
// 17:16 Reserved.
// 15 TCE: translation cache extension enable. Read-write. Reset: 0. 1=Translation cache extension is enabled. PDC
// entries related to the linear address of the INVLPG instruction are invalidated. If this bit is 0 all PDC entries are
// invalidated by the INVLPG instruction.
// 14 FFXSE: fast FXSAVE/FRSTOR enable. Read-write. Reset: 0. 1=Enables the fast FXSAVE/FRSTOR
// mechanism. A 64-bit operating system may enable the fast FXSAVE/FRSTOR mechanism if
// (Core::X86::Cpuid::FeatureExtIdEdx[FFXSR] == 1). This bit is set once by the operating system and its value is
// not changed afterwards.
// 13 LMSLE: long mode segment limit enable. Read-only,Error-on-write-1. Reset: Fixed,0. 1=Enables the long
// mode segment limit check mechanism.
// 12 SVME: secure virtual machine (SVM) enable. Reset: Fixed,0. 1=SVM features are enabled.
// AccessType: Core::X86::Msr::VM_CR[SvmeDisable] ? Read-only,Error-on-write-1 : Read-write.
// 11 NXE: no-execute page enable. Read-write. Reset: 0. 1=The no-execute page protection feature is enabled.
// 10 LMA: long mode active. Read-only. Reset: 0. 1=Indicates that long mode is active. When writing the EFER
// register the value of this bit must be preserved. Software must read the EFER register to determine the value of
// LMA, change any other bits as required and then write the EFER register. An attempt to write a value that differs
// from the state determined by hardware results in a #GP fault.
// 9 Reserved.
// 8 LME: long mode enable. Read-write. Reset: 0. 1=Long mode is enabled.
// 7:1 Reserved.
// 0 SYSCALL: system call extension enable. Read-write. Reset: 0. 1=SYSCALL and SYSRET instructions are
// enabled. This adds the SYSCALL and SYSRET instructions which can be used in flat addressed operating
// systems as low latency system calls and returns.
//
// MSRC000_0108 [Prefetch Control] (Core::X86::Msr::PrefetchControl)
// Reset: 0000_0000_0000_0000h.
// _ccd[11:0]_lthree0_core[7:0]; MSRC000_0108
// Bits Description
// 63:6 Reserved.
// 5 UpDown. Read-write. Reset: 0. Disable prefetcher that uses memory access history to determine whether to fetch
// the next or previous line into L2 cache for all memory accesses.
// 4 Reserved.
// 3 L2Stream. Read-write. Reset: 0. Disable prefetcher that uses history of memory access patterns to fetch
// additional sequential lines into L2 cache.
// 2 L1Region. Read-write. Reset: 0. Disable prefetcher that uses memory access history to fetch additional lines into
// L1 cache when the data access for a given instruction tends to be followed by a consistent pattern of other
// accesses within a localized region.
// 1 L1Stride. Read-write. Reset: 0. Disable stride prefetcher that uses memory access history of individual
// instructions to fetch additional lines into L1 cache when each access is a constant distance from the previous.
// 0 L1Stream. Read-write. Reset: 0. Disable stream prefetcher that uses history of memory access patterns to fetch
// additional sequential lines into L1 cache.
//
// MSRC000_010E [LBR V2 Branch Select] (Core::X86::Msr::LbrSelect)
// Read-write. Reset: 0000_0000_0000_0000h.
// This MSR allows LBR V2 recording to be suppressed based on branch type and privilege level.
// _ccd[11:0]_lthree0_core[7:0]_thread[1:0]; MSRC000_010E
// Bits Description
// 63:9 Reserved.
// 8 FarBranch. Read-write. Reset: 0. When set far branches are not recorded.
// 7 JmpNearRel. Read-write. Reset: 0. When set, near relative jumps, excluding near relative calls, are not recorded.
// 6 JmpNearInd. Read-write. Reset: 0. When set, near indirect jumps, excluding near indirect calls and near returns,
// are not recorded.
// 5 RetNear. Read-write. Reset: 0. When set, near returns are not recorded.
// 4 CallNearInd. Read-write. Reset: 0. When set, near indirect calls are not recorded.
// 3 CallNearRel. Read-write. Reset: 0. When set, near relative calls are not recorded.
// 2 Jcc. Read-write. Reset: 0. When set conditional branches are not recorded.
// 1 CplGe0. Read-write. Reset: 0. When set, no branches ending in CPL > 0 are recorded.
// 0 CplEq0. Read-write. Reset: 0. When set, no branches ending in CPL = 0 are recorded.
//
// MSRC001_0010 [System Configuration] (Core::X86::Msr::SYS_CFG)
// Reset: 0000_0000_0000_0000h.
// If Core::X86::Msr::SYS_CFG[SecureNestedPagingEn] is set, writes to this register are ignored.
// _ccd[11:0]_lthree0_core[7:0]; MSRC001_0010
// Bits Description
// 63:27 Reserved.
// 26 HMKEE: Host Multi-Key Encryption Enable. Read,Write-1-only. Reset: 0. Used with SYS_CFG[SMEE] to
// select secure memory encryption mode. See SYS_CFG[SMEE] for a table listing the available memory
// encryption modes.
// 25 VmplEn. Reset: 0. VM permission levels enable.
// AccessType: Core::X86::Msr::SYS_CFG[SecureNestedPagingEn] ? Read-only : Read-write.
// 24 SecureNestedPagingEn. Read,Write-1-only. Reset: 0. Enable Secure Nested Paging (SNP).
// 23 SMEE: Secure Memory Encryption Enable. Read,Write-1-only. Reset: 0.
// Description: Used with SYS_CFG[HMKEE] to select secure memory encryption mode. See the table below for
// the available memory encryption modes.
// HMKEE SMEE Description
// 0 0 No encryption.
// 0 1 Enables SME and SEV memory encryption.
// 1 0 Enables SME-HMK memory encryption.
// 1 1 Not supported. Results in #GP.
// 22 Tom2ForceMemTypeWB: top of memory 2 memory type write back. Read-write. Reset: 0. 1=The default
// memory type of memory between 4GB and Core::X86::Msr::TOM2 is write back instead of the memory type
// defined by Core::X86::Msr::MTRRdefType[MemType]. For this bit to have any effect,
// Core::X86::Msr::MTRRdefType[MtrrDefTypeEn] must be 1. MTRRs and PAT can be used to override this
// memory type.
// 21 MtrrTom2En: MTRR top of memory 2 enable. Read-write. Reset: 0. 0=Core::X86::Msr::TOM2 is disabled. 1=
// Core::X86::Msr::TOM2 is enabled.
// 20 MtrrVarDramEn: MTRR variable DRAM enable. Read-write. Reset: 0. Init: BIOS,1.
// 0=Core::X86::Msr::TOP_MEM and IORRs are disabled. 1=These registers are enabled.
// 19 MtrrFixDramModEn: MTRR fixed RdDram and WrDram modification enable. Read-write. Reset: 0.
// 0=Core::X86::Msr::MtrrFix_64K through Core::X86::Msr::MtrrFix_4K_7 [RdDram,WrDram] read values is
// masked 00b; writing does not change the hidden value. 1=Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7 [RdDram,WrDram] access type is Read-write. Not shared between threads.
// Controls access to Core::X86::Msr::MtrrFix_64K through Core::X86::Msr::MtrrFix_4K_7 [RdDram ,WrDram].
// This bit should be set to 1 during BIOS initialization of the fixed MTRRs, then cleared to 0 for operation.
// 18 MtrrFixDramEn: MTRR fixed RdDram and WrDram attributes enable. Read-write. Reset: 0. Init: BIOS,1.
// 1=Enables the RdDram and WrDram attributes in Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7.
// 17:0 Reserved.
//
// MSRC001_0015 [Hardware Configuration] (Core::X86::Msr::HWCR)
// Reset: 0000_0000_0100_0010h.
// _ccd[11:0]_lthree0_core[7:0]_thread[1:0]; MSRC001_0015
// Bits Description
// 63:36 Reserved.
// 35 CpuidFltEn. Read-write. Reset: 0. 1=Executing CPUID outside of SMM and with CPL > 0 results in #GP.
// 34 Reserved.
// 33 SmmPgCfgLock. Read-write. Reset: 0. 1=SMM page config locked. Error-on-write-1 if not in SMM mode. RSM
// unconditionally clears Core::X86::Msr::HWCR[SmmPgCfgLock].
// 32:31 Reserved.
// 30 IRPerfEn: enable instructions retired counter. Read-write. Reset: 0. 1=Enable Core::X86::Msr::IRPerfCount.
// 29:28 Reserved.
// 27 EffFreqReadOnlyLock: read-only effective frequency counter lock. Write-1-only. Reset: 0. Init: BIOS,1.
// 1=Core::X86::Msr::MPerfReadOnly, Core::X86::Msr::APerfReadOnly and Core::X86::Msr::IRPerfCount are
// read-only.
// 26 EffFreqCntMwait: effective frequency counting during mwait. Read-write. Reset: 0. 0=The registers do not
// increment. 1=The registers increment. Specifies whether Core::X86::Msr::MPERF and Core::X86::Msr::APERF
// increment while the core is in the monitor event pending state. See 2.1.6 [Effective Frequency].
// 25 CpbDis: core performance boost disable. Read-write. Reset: 0. 0=CPB is requested to be enabled. 1=CPB is
// disabled. Specifies whether core performance boost is requested to be enabled or disabled. If core performance
// boost is disabled while a core is in a boosted P-state, the core automatically transitions to the highest performance
// non-boosted P-state.
// 24 TscFreqSel: TSC frequency select. Read-only. Reset: 1. 1=The TSC increments at the P0 frequency.
// 23:22 Reserved.
// 21 LockTscToCurrentP0: lock the TSC to the current P0 frequency. Read-write. Reset: 0. 0=The TSC will count
// at the P0 frequency. 1=The TSC frequency is locked to the current P0 frequency at the time this bit is set and
// remains fixed regardless of future changes to the P0 frequency.
// 20 IoCfgGpFault: IO-space configuration causes a GP fault. Read-write. Reset: 0. 1=IO-space accesses to
// configuration space cause a GP fault. The fault is triggered if any part of the IO Read/Write address range is
// between CF8h and CFFh, inclusive. These faults only result from single IO instructions, not to string and REP IO
// instructions. This fault takes priority over the IO trap mechanism described by
// Core::X86::Msr::SMI_ON_IO_TRAP_CTL_STS.
// 19 Reserved.
// 18 McStatusWrEn: machine check status write enable. Read-write. Reset: 0. 0=MCA_STATUS registers are
// readable; writing a non-zero pattern to these registers causes a general protection fault. 1=MCA_STATUS
// registers are Read-write, including Reserved fields; do not cause general protection faults; such writes update all
// implemented bits in these registers; All fields of all threshold registers are Read-write when accessed from MSR
// space, including Locked, except BlkPtr which is always Read-only; McStatusWrEn does not change the access
// type for the thresholding registers accessed via configuration space.
// Description: McStatusWrEn can be used to debug machine check exception and interrupt handlers.
// Independent of the value of this bit, the processor may enforce Write-Ignored behavior on MCA_STATUS
// registers depending on platform settings.
// See 3.1 [Machine Check Architecture].
// 17 Wrap32Dis: 32-bit address wrap disable. Read-write. Reset: 0. 1=Disable 32-bit address wrapping. Software
// can use Wrap32Dis to access physical memory above 4 Gbytes without switching into 64-bit mode. To do so,
// software should write a greater-than 4 Gbyte address to Core::X86::Msr::FS_BASE and
// Core::X86::Msr::GS_BASE. Then it would address ±2 Gbytes from one of those bases using normal memory
// reference instructions with a FS or GS override prefix. However, the INVLPG, FST, and SSE store instructions
// generate 32-bit addresses in legacy mode, regardless of the state of Wrap32Dis.
// 16:15 Reserved.
// 14 RsmSpCycDis: RSM special bus cycle disable. Reset: 0. Init: BIOS,1. 0=A link special bus cycle, SMIACK, is
// generated on a resume from SMI.
// AccessType: Core::X86::Msr::HWCR[SmmLock] ? Read-only : Read-write.
// 13 SmiSpCycDis: SMI special bus cycle disable. Reset: 0. Init: BIOS,1. 0=A link special bus cycle, SMIACK, is
// generated when an SMI interrupt is taken.
// AccessType: Core::X86::Msr::HWCR[SmmLock] ? Read-only : Read-write.
// 12:11 Reserved.
// 10 MonMwaitUserEn: MONITOR/MWAIT user mode enable. Read-write. Reset: 0. 0=The MONITOR and
// MWAIT instructions are supported only in privilege level 0; these instructions in privilege levels 1 to 3 cause a
// #UD exception. 1=The MONITOR and MWAIT instructions are supported in all privilege levels. The state of this
// bit is ignored if MonMwaitDis is set.
// 9 MonMwaitDis: MONITOR and MWAIT disable. Read-write. Reset: 0. 1=The MONITOR, MWAIT,
// MONITORX, and MWAITX opcodes become invalid. This affects what is reported back through
// Core::X86::Cpuid::FeatureIdEcx[Monitor] and Core::X86::Cpuid::FeatureExtIdEcx[MwaitExtended].
// 8 IgnneEm: IGNNE port emulation enable. Read-write. Reset: 0. 1=Enable emulation of IGNNE port.
// 7 AllowFerrOnNe: allow FERR on NE. Read-write. Reset: 0. 0=Disable FERR signalling when generating an x87
// floating point exception (when CR0[NE] is set). 1=FERR is signaled on any x87 floating point exception,
// regardless of CR0[NE].
// 6:5 Reserved.
// 4 INVDWBINVD: INVD to WBINVD conversion. Read,Error-on-write-0. Reset: 1. 1=Convert INVD to
// WBINVD.
// Description: This bit is required to be set for normal operation when any of the following are true:
// • An L2 is shared by multiple threads.
// • An L3 is shared by multiple cores.
// • CC6 is enabled.
// • Probe filter is enabled.
// 3 TlbCacheDis: cacheable memory disable. Read-write. Reset: 0. 1=Disable performance improvement that
// assumes that the PML4, PDP, PDE and PTE entries are in cacheable WB DRAM.
// Description: Operating systems that maintain page tables in any other memory type must set the TlbCacheDis bit
// to insure proper operation. Operating system should do a full TLB flush before and after any changes to this bit
// value.
// • TlbCacheDis does not override the memory type specified by the SMM ASeg and TSeg memory regions
// controlled by Core::X86::Msr::SMMAddr Core::X86::Msr::SMMMask.
// 2:1 Reserved.
// 0 SmmLock: SMM code lock. Read,Write-1-only. Reset: 0. Init: BIOS,1. 1=SMM code in the ASeg and TSeg
// range and the SMM registers are Read-only and SMI interrupts are not intercepted in SVM. See 2.1.13.1.10
// [Locking SMM].
//
// MSRC001_02B1 [CPPC Enable] (Core::X86::Msr::CppcEnable)
// Collaborative Processor Performance Control Enable.
// _ccd[11:0]_lthree0_core[7:0]_thread[1:0]; MSRC001_02B1
// Bits Description
// 63:1 Reserved.
// 0 CppcEnable. Read,Write-1-only. Reset: 0. CPPC Enable.
//
// MSR0000_0C81 [L3 QoS Configuration] (L3::L3CRB::L3QosCfg1)
// Reset: 0000_0000_0000_0000h.
// QOS L3 Cache Allocation CDP mode enable (I vs. D). Contents are copied to ChL2QosCfg1 and ChL3QosCfg1_0.
// _ccd[11:0]_lthree0; MSR0000_0C81
// Bits Description
// 63:1 Reserved.
// 0 CDP. Read-write. Reset: 0. Code and Data Prioritization Technology enable

// AMD MSR Documentation (Zen5)
//
// MSR0000_0048 [Speculative Control] (Core::X86::Msr::SPEC_CTRL)
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSR0000_0048
// Bits Description
// 63:8 Reserved.
// 7 PSFD: Predictive Store Forwarding Disable. Read-write. Reset: 0. 1=Disable predictive store forwarding.
// 6:3 Reserved.
// 2 SSBD. Read-write. Reset: 0. Speculative Store Bypass Disable.
// 1 STIBP. Read-write. Reset: 0. Single thread indirect branch predictor.
// 0 IBRS. Read-write. Reset: 0. Indirect branch restriction speculation.
//
// MSR0000_0049 [Prediction Command] (Core::X86::Msr::PRED_CMD)
// Write-only,Error-on-read. Reset: 0000_0000_0000_0000h.
// _ccd[11:0]_lthree0_core[15:0]; MSR0000_0049
// Bits Description
// 63:8 Reserved.
// 7 SBPB: selective branch predictor barrior. Write-only,Error-on-read. Reset: 0. When SBPB is supported
// (Core::X86::Cpuid::FeatureExt2Eax[SBPB]==1), setting this bit initiates a selective branch predictor barrier
// 6:1 Reserved.
// 0 IBPB: indirect branch prediction barrier. Write-only,Error-on-read. Reset: 0. Supported if
// Core::X86::Cpuid::FeatureExtIdEbx[IBPB] == 1.
//
// MSR0000_0049 [Prediction Command] (Core::X86::Msr::PRED_CMD)
// Write-only,Error-on-read. Reset: 0000_0000_0000_0000h.
// _ccd[11:0]_lthree0_core[15:0]; MSR0000_0049
// Bits Description
// 63:8 Reserved.
// 7 SBPB: selective branch predictor barrior. Write-only,Error-on-read. Reset: 0. When SBPB is supported
// (Core::X86::Cpuid::FeatureExt2Eax[SBPB]==1), setting this bit initiates a selective branch predictor barrier
// 6:1 Reserved.
// 0 IBPB: indirect branch prediction barrier. Write-only,Error-on-read. Reset: 0. Supported if
// Core::X86::Cpuid::FeatureExtIdEbx[IBPB] == 1.
//
// MSRC000_0080 [Extended Feature Enable] (Core::X86::Msr::EFER)
// SKINIT Execution: 0000_0000_0000_0000h.
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSRC000_0080
// Bits Description
// 63:22 Reserved.
// 21 AutomaticIBRSEn: Automatic IBRS Enable. Read-write. Reset: 0. 0=IBRS protection is not enabled unless
// (SPEC_CTRL[IBRS] == 1). 1=IBRS protection is enabled for any process running at (CPL == 0) or ((ASID ==
// 0) && SEV-SNP).
// 20 UAIE: Upper Address Ignore Enable. Read-write. Reset: 0. Upper Address Ignore suppresses canonical faults
// for most data access virtual addresses, which allows software to use the upper bits of a virtual address as tags.
// 19 Reserved.
// 18 IntWbinvdEn. Read-write. Reset: 0. Interruptible wbinvd, wbnoinvd enable.
// 17:16 Reserved.
// 15 TCE: translation cache extension enable. Read-write. Reset: 0. 1=Translation cache extension is enabled. PDC
// entries related to the linear address of the INVLPG instruction are invalidated. If this bit is 0 all PDC entries are
// invalidated by the INVLPG instruction.
// 14 FFXSE: fast FXSAVE/FRSTOR enable. Read-write. Reset: 0. 1=Enables the fast FXSAVE/FRSTOR
// mechanism. A 64-bit operating system may enable the fast FXSAVE/FRSTOR mechanism if
// (Core::X86::Cpuid::FeatureExtIdEdx[FFXSR] == 1). This bit is set once by the operating system and its value is
// not changed afterwards.
// 13 LMSLE: long mode segment limit enable. Read-only,Error-on-write-1. Reset: Fixed,0. 1=Enables the long
// mode segment limit check mechanism.
// 12 SVME: secure virtual machine (SVM) enable. Reset: Fixed,0. 1=SVM features are enabled.
// AccessType: Core::X86::Msr::VM_CR[SvmeDisable] ? Read-only,Error-on-write-1 : Read-write.
// 11 NXE: no-execute page enable. Read-write. Reset: 0. 1=The no-execute page protection feature is enabled.
// 10 LMA: long mode active. Read-only. Reset: 0. 1=Indicates that long mode is active. When writing the EFER
// register the value of this bit must be preserved. Software must read the EFER register to determine the value of
// LMA, change any other bits as required and then write the EFER register. An attempt to write a value that differs
// from the state determined by hardware results in a #GP fault.
// 9 Reserved.
// 8 LME: long mode enable. Read-write. Reset: 0. 1=Long mode is enabled.
// 7:1 Reserved.
// 0 SYSCALL: system call extension enable. Read-write. Reset: 0. 1=SYSCALL and SYSRET instructions are
// enabled. This adds the SYSCALL and SYSRET instructions which can be used in flat addressed operating
// systems as low latency system calls and returns.
//
// MSRC000_0108 [Prefetch Control] (Core::X86::Msr::PrefetchControl)
// Reset: 0000_0000_0000_03C0h.
// _ccd[11:0]_lthree0_core[15:0]; MSRC000_0108
// Bits Description
// 63:10 Reserved.
// 9:7 PrefetchAggressivenessProfile. Read-write. Reset: 7h. When MasterEnable is set, selects a prefetch
// aggressiveness profile.
// ValidValues:
// Value Description
// 0h Level 0, least aggressive prefetch profile.
// 1h Level 1
// 2h Level 2
// 3h Level 3, most aggressive prefetch profile.
// 6h-4h Reserved.
// 7h Default used by hardware. Not software accessible.
// 6 MasterEnable. Read-write. Reset: 1. Enable prefetch aggressiveness profiles.
// 5 UpDown. Read-write. Reset: 0. Disable prefetcher that uses memory access history to determine whether to fetch
// the next or previous line into L2 cache for all memory accesses.
// 4 Reserved.
// 3 L2Stream. Read-write. Reset: 0. Disable prefetcher that uses history of memory access patterns to fetch
// additional sequential lines into L2 cache.
// 2 L1Region. Read-write. Reset: 0. Disable prefetcher that uses memory access history to fetch additional lines into
// L1 cache when the data access for a given instruction tends to be followed by a consistent pattern of other
// accesses within a localized region.
// 1 L1Stride. Read-write. Reset: 0. Disable stride prefetcher that uses memory access history of individual
// instructions to fetch additional lines into L1 cache when each access is a constant distance from the previous.
// 0 L1Stream. Read-write. Reset: 0. Disable stream prefetcher that uses history of memory access patterns to fetch
// additional sequential lines into L1 cache.
//
// MSRC001_0010 [System Configuration] (Core::X86::Msr::SYS_CFG)
// Reset: 0000_0000_0000_0000h.
// If Core::X86::Msr::SYS_CFG[SecureNestedPagingEn] is set, writes to this register are ignored.
// _ccd[11:0]_lthree0_core[15:0]; MSRC001_0010
// Bits Description
// 63:27 Reserved.
// 26 HMKEE: Host Multi-Key Encryption Enable. Read,Write-1-only. Reset: 0. Used with SYS_CFG[SMEE] to
// select secure memory encryption mode. See SYS_CFG[SMEE] for a table listing the available memory
// encryption modes.
// 25 VmplEn. Reset: 0. VM permission levels enable.
// AccessType: Core::X86::Msr::SYS_CFG[SecureNestedPagingEn] ? Read-only : Read-write.
// 24 SecureNestedPagingEn. Read,Write-1-only. Reset: 0. Enable Secure Nested Paging (SNP).
// 23 SMEE: Secure Memory Encryption Enable. Read,Write-1-only. Reset: 0.
// Description: Used with SYS_CFG[HMKEE] to select secure memory encryption mode. See the table below for
// the available memory encryption modes.
// HMKEE SMEE Description
// 0 0 No encryption.
// 0 1 Enables SME and SEV memory encryption.
// 1 0 Enables SME-HMK memory encryption.
// 1 1 Not supported. Results in #GP.
// 22 Tom2ForceMemTypeWB: top of memory 2 memory type write back. Read-write. Reset: 0. 1=The default
// memory type of memory between 4GB and Core::X86::Msr::TOM2 is write back instead of the memory type
// defined by Core::X86::Msr::MTRRdefType[MemType]. For this bit to have any effect,
// Core::X86::Msr::MTRRdefType[MtrrDefTypeEn] must be 1. MTRRs and PAT can be used to override this
// memory type.
// 21 MtrrTom2En: MTRR top of memory 2 enable. Read-write. Reset: 0. 0=Core::X86::Msr::TOM2 is disabled. 1=
// Core::X86::Msr::TOM2 is enabled.
// 20 MtrrVarDramEn: MTRR variable DRAM enable. Read-write. Reset: 0. Init: BIOS,1.
// 0=Core::X86::Msr::TOP_MEM and IORRs are disabled. 1=These registers are enabled.
// 19 MtrrFixDramModEn: MTRR fixed RdDram and WrDram modification enable. Read-write. Reset: 0.
// 0=Core::X86::Msr::MtrrFix_64K through Core::X86::Msr::MtrrFix_4K_7 [RdDram,WrDram] read values is
// masked 00b; writing does not change the hidden value. 1=Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7 [RdDram,WrDram] access type is Read-write. Not shared between threads.
// Controls access to Core::X86::Msr::MtrrFix_64K through Core::X86::Msr::MtrrFix_4K_7 [RdDram ,WrDram].
// This bit should be set to 1 during BIOS initialization of the fixed MTRRs, then cleared to 0 for operation.
// 18 MtrrFixDramEn: MTRR fixed RdDram and WrDram attributes enable. Read-write. Reset: 0. Init: BIOS,1.
// 1=Enables the RdDram and WrDram attributes in Core::X86::Msr::MtrrFix_64K through
// Core::X86::Msr::MtrrFix_4K_7.
// 17:0 Reserved.
// MSRC001_0015 [Hardware Configuration] (Core::X86::Msr::HWCR)
// Reset: 0000_0000_0100_6010h.
// _ccd[11:0]_lthree0_core[15:0]_thread[1:0]; MSRC001_0015
// Bits Description
// 63:36 Reserved.
// 35 CpuidFltEn. Read-write. Reset: 0. 1=Executing CPUID outside of SMM and with CPL > 0 results in #GP.
// 34 DownGradeFp512ToFP256. Read-write. Reset: 0. 1=Downgrade FP512 performance to look more like FP256
// performance.
// 33 SmmPgCfgLock. Read-write. Reset: 0. 1=SMM page config locked. Error-on-write-1 if not in SMM mode. RSM
// unconditionally clears Core::X86::Msr::HWCR[SmmPgCfgLock].
// 32:31 Reserved.
// 30 IRPerfEn: enable instructions retired counter. Read-write. Reset: 0. 1=Enable Core::X86::Msr::IRPerfCount.
// 29:28 Reserved.
// 27 EffFreqReadOnlyLock: read-only effective frequency counter lock. Write-1-only. Reset: 0. Init: BIOS,1.
// 1=Core::X86::Msr::MPerfReadOnly, Core::X86::Msr::APerfReadOnly and Core::X86::Msr::IRPerfCount are
// read-only.
// 26 EffFreqCntMwait: effective frequency counting during mwait. Read-write. Reset: 0. 0=The registers do not
// increment. 1=The registers increment. Specifies whether Core::X86::Msr::MPERF and Core::X86::Msr::APERF
// increment while the core is in the monitor event pending state. See 2.1.6 [Effective Frequency].
// 25 CpbDis: core performance boost disable. Read-write. Reset: 0. 0=CPB is requested to be enabled. 1=CPB is
// disabled. Specifies whether core performance boost is requested to be enabled or disabled. If core performance
// boost is disabled while a core is in a boosted P-state, the core automatically transitions to the highest performance
// non-boosted P-state.
// 24 TscFreqSel: TSC frequency select. Read-only. Reset: 1. 1=The TSC increments at the P0 frequency.
// 23:22 Reserved.
// 21 LockTscToCurrentP0: lock the TSC to the current P0 frequency. Read-write. Reset: 0. 0=The TSC will count
// at the P0 frequency. 1=The TSC frequency is locked to the current P0 frequency at the time this bit is set and
// remains fixed regardless of future changes to the P0 frequency.
// 20 IoCfgGpFault: IO-space configuration causes a GP fault. Read-write. Reset: 0. 1=IO-space accesses to
// configuration space cause a GP fault. The fault is triggered if any part of the IO Read/Write address range is
// between CF8h and CFFh, inclusive. These faults only result from single IO instructions, not to string and REP IO
// instructions. This fault takes priority over the IO trap mechanism described by
// Core::X86::Msr::SMI_ON_IO_TRAP_CTL_STS.
// 19 Reserved.
// 18 McStatusWrEn: machine check status write enable. Read-write. Reset: 0. 0=MCA_STATUS registers are
// readable; writing a non-zero pattern to these registers causes a general protection fault. 1=MCA_STATUS
// registers are Read-write, including Reserved fields; do not cause general protection faults; such writes update all
// implemented bits in these registers; All fields of all threshold registers are Read-write when accessed from MSR
// space, including Locked, except BlkPtr which is always Read-only; McStatusWrEn does not change the access
// type for the thresholding registers accessed via configuration space.
// Description: McStatusWrEn can be used to debug machine check exception and interrupt handlers.
// Independent of the value of this bit, the processor may enforce Write-Ignored behavior on MCA_STATUS
// registers depending on platform settings.
// See 3.1 [Machine Check Architecture].
// 17 Wrap32Dis: 32-bit address wrap disable. Read-write. Reset: 0. 1=Disable 32-bit address wrapping. Software
// can use Wrap32Dis to access physical memory above 4 Gbytes without switching into 64-bit mode. To do so,
// software should write a greater-than 4 Gbyte address to Core::X86::Msr::FS_BASE and
// Core::X86::Msr::GS_BASE. Then it would address ±2 Gbytes from one of those bases using normal memory
// reference instructions with a FS or GS override prefix. However, the INVLPG, FST, and SSE store instructions
// generate 32-bit addresses in legacy mode, regardless of the state of Wrap32Dis.
// 16:15 Reserved.
// 14 RsmSpCycDis: RSM special bus cycle disable. Reset: 1. Init: BIOS,1. 0=A link special bus cycle, SMIACK, is
// generated on a resume from SMI.
// AccessType: Core::X86::Msr::HWCR[SmmLock] ? Read-only : Read-write.
// 13 SmiSpCycDis: SMI special bus cycle disable. Reset: 1. Init: BIOS,1. 0=A link special bus cycle, SMIACK, is
// generated when an SMI interrupt is taken.
// AccessType: Core::X86::Msr::HWCR[SmmLock] ? Read-only : Read-write.
// 12:11 Reserved.
// 10 MonMwaitUserEn: MONITOR/MWAIT user mode enable. Read-write. Reset: 0. 0=The MONITOR and
// MWAIT instructions are supported only in privilege level 0; these instructions in privilege levels 1 to 3 cause a
// #UD exception. 1=The MONITOR and MWAIT instructions are supported in all privilege levels. The state of this
// bit is ignored if MonMwaitDis is set.
// 9 MonMwaitDis: MONITOR and MWAIT disable. Read-write. Reset: 0. 1=The MONITOR, MWAIT,
// MONITORX, and MWAITX opcodes become invalid. This affects what is reported back through
// Core::X86::Cpuid::FeatureIdEcx[Monitor] and Core::X86::Cpuid::FeatureExtIdEcx[MwaitExtended].
// 8 IgnneEm: IGNNE port emulation enable. Read-write. Reset: 0. 1=Enable emulation of IGNNE port.
// 7 AllowFerrOnNe: allow FERR on NE. Read-write. Reset: 0. 0=Disable FERR signalling when generating an x87
// floating point exception (when CR0[NE] is set). 1=FERR is signaled on any x87 floating point exception,
// regardless of CR0[NE].
// 6:5 Reserved.
// 4 INVDWBINVD: INVD to WBINVD conversion. Read,Error-on-write-0. Reset: 1. 1=Convert INVD to
// WBINVD.
// Description: This bit is required to be set for normal operation when any of the following are true:
// • An L2 is shared by multiple threads.
// • An L3 is shared by multiple cores.
// • CC6 is enabled.
// • Probe filter is enabled.
// 3 TlbCacheDis: cacheable memory disable. Read-write. Reset: 0. 1=Disable performance improvement that
// assumes that the PML4, PDP, PDE and PTE entries are in cacheable WB DRAM.
// Description: Operating systems that maintain page tables in any other memory type must set the TlbCacheDis bit
// to insure proper operation. Operating system should do a full TLB flush before and after any changes to this bit
// value.
// • TlbCacheDis does not override the memory type specified by the SMM ASeg and TSeg memory regions
// controlled by Core::X86::Msr::SMMAddr Core::X86::Msr::SMMMask.
// 2:1 Reserved.
// 0 SmmLock: SMM code lock. Read,Write-1-only. Reset: 0. Init: BIOS,1. 1=SMM code in the ASeg and TSeg
// range and the SMM registers are Read-only and SMI interrupts are not intercepted in SVM. See 2.1.13.1.10
// [Locking SMM].
//
// MSRC001_02B1 [CPPC Enable] (Core::X86::Msr::CppcEnable)
// Collaborative Processor Performance Control Enable.
// MSRC001_02B1
// Bits Description
// 63:1 Reserved.
// 0 CppcEnable. Read,Write-1-only. Reset: 0. CPPC Enable.
