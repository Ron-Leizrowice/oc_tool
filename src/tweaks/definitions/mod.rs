use std::collections::BTreeMap;

use disable_processor_idle_states::DisableProcessIdleStates;
use kill_explorer::KillExplorerTweak;
use kill_non_critical_services::KillNonCriticalServicesTweak;
use low_res_mode::LowResMode;
use process_idle_tasks::ProcessIdleTasksTweak;

use super::{
    group_policy::{GroupPolicyTweak, GroupPolicyValue},
    powershell::PowershellTweak,
    registry::{RegistryKeyValue, RegistryModification, RegistryTweak},
    Tweak, TweakCategory,
};
use crate::widgets::TweakWidget;

pub mod disable_processor_idle_states;
pub mod kill_explorer;
pub mod kill_non_critical_services;
pub mod low_res_mode;
pub mod process_idle_tasks;

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
}

/// Initializes all tweaks with their respective configurations.
pub fn all() -> BTreeMap<TweakId, Tweak> {
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
    ])
}

pub fn disable_process_idle_states() -> Tweak {
    Tweak::rust_tweak(
        "Disable Process Idle States".to_string(),
        "Disables processor idle states (C-states) to prevent the CPU from entering low-power states during idle periods. This tweak can improve system responsiveness but may increase power consumption and heat output.".to_string(),
        TweakCategory::Power,
        DisableProcessIdleStates::new(),
        TweakWidget::Toggle,
        false,
    )
}

pub fn process_idle_tasks() -> Tweak {
    Tweak::rust_tweak(
        "Process Idle Tasks".to_string(),
        "Forces the execution of scheduled background tasks that are normally run during system idle time. This helps free up system resources by completing these tasks immediately, improving overall system responsiveness and optimizing resource allocation. It can also reduce latency caused by deferred operations in critical system processes.".to_string(),
        TweakCategory::Action,
        ProcessIdleTasksTweak{
            id: TweakId::ProcessIdleTasks,
        },
        TweakWidget::Button,
        false,
    )
}

pub fn kill_all_non_critical_services() -> Tweak {
    Tweak::rust_tweak(
        "Kill All Non-Critical Services".to_string(),
        "Stops all non-critical services to free up system resources and improve performance. This tweak may cause system instability or data loss.".to_string(),
        TweakCategory::Services,
        KillNonCriticalServicesTweak {
            id: TweakId::KillAllNonCriticalServices,
        },
        TweakWidget::Button,
        false,
    )
}

/// Initializes the Kill Explorer tweak.
pub fn kill_explorer() -> Tweak {
    Tweak::rust_tweak(
        "Kill Explorer".to_string(),
        "Terminates the Windows Explorer process and prevents it from automatically restarting. This can free up system resources but will remove the desktop interface. Use with caution.".to_string(),
        TweakCategory::Action,
        KillExplorerTweak {
            id: TweakId::KillExplorer,
        },
        TweakWidget::Toggle,
        false,
    )
}

pub fn ultimate_performance_plan() -> Tweak {
    Tweak::powershell_tweak(
        "Enable Ultimate Performance Plan".to_string(),
        "Activates the Ultimate Performance power plan, which is tailored for demanding workloads by minimizing micro-latencies and boosting hardware performance. It disables power-saving features like core parking, hard disk sleep, and processor throttling, ensuring CPU cores run at maximum frequency. This plan also keeps I/O devices and PCIe links at full power, prioritizing performance over energy efficiency. It’s designed to reduce the delays introduced by energy-saving policies, improving responsiveness in tasks that require consistent, high-throughput system resources..".to_string(),
        TweakCategory::Power,
        PowershellTweak {
            id: TweakId::UltimatePerformancePlan,
            read_script: Some(
                "powercfg /GETACTIVESCHEME".to_string(),
            ),
            apply_script:
                r#"
                powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61
                $ultimatePlans = powercfg /L | Select-String '(Ultimate Performance)' | ForEach-Object { $_.Line }
                $ultimatePlans = @($ultimatePlans | ForEach-Object { $_ -replace 'Power Scheme GUID: ', '' -replace ' \(Ultimate Performance\)', '' -replace '\*$', '' } | ForEach-Object { $_.Trim() })
                for ($i = 0; $i -lt $ultimatePlans.Length - 1; $i++) { powercfg -delete $ultimatePlans[$i] }
                powercfg /SETACTIVE $ultimatePlans[-1]
                "#
                .trim()
                .to_string()
            ,
            undo_script: Some(
                r#"
                $balancedPlan = powercfg /L | Select-String '(Balanced)' | ForEach-Object { $_.Line }
                $balancedPlan = $balancedPlan -replace 'Power Scheme GUID: ', '' -replace ' \(Balanced\)', '' -replace '\*$', '' | ForEach-Object { $_.Trim() }
                powercfg /S $balancedPlan
                $ultimatePlans = powercfg /L | Select-String '(Ultimate Performance)' | ForEach-Object { $_.Line }
                $ultimatePlans = @($ultimatePlans | ForEach-Object { $_ -replace 'Power Scheme GUID: ', '' -replace ' \(Ultimate Performance\)', '' -replace '\*$', '' } | ForEach-Object { $_.Trim() })
                foreach ($plan in $ultimatePlans) { powercfg -delete $plan }
                "#
                .trim()
                .to_string(),
            ),
            target_state: Some("(Ultimate Performance)".trim().to_string()),
        },
        false, // requires reboot

    )
}

pub fn disable_hpet() -> Tweak {
    Tweak::powershell_tweak(
        "Disable Dynamic Tick".to_string(),
        "Disables the dynamic tick feature, which normally reduces timer interrupts during idle periods to conserve power. By disabling dynamic tick, the system maintains a constant rate of timer interrupts, improving performance in real-time applications by reducing latency and jitter. This tweak is useful in scenarios where consistent, low-latency processing is required, but it may increase power consumption as the CPU will not enter low-power states as frequently.".to_string(),
        TweakCategory::System,
        PowershellTweak {
            id: TweakId::DisableHPET,
            read_script: Some(r#"(bcdedit /enum | Select-String "useplatformclock").ToString().Trim()"#.to_string()),

            apply_script: r#"
            bcdedit /deletevalue useplatformclock
            bcdedit /set disabledynamictick yes
            "#.trim().to_string(),

            undo_script: Some(r#"
            bcdedit /set useplatformclock true
            bcdedit /set disabledynamictick no
            "#.trim().to_string()),

            target_state: Some("useplatformclock        Yes".trim().to_string()),
        },
        true,
    )
}

pub fn disable_ram_compression() -> Tweak {
    Tweak::powershell_tweak(
        "Disable RAM Compression".to_string(),
        "Disables the RAM compression feature in Windows to potentially improve system performance by reducing CPU overhead. This may lead to higher memory usage.".to_string(),
        TweakCategory::Memory,
        PowershellTweak {
            id: TweakId::DisableRamCompression,
            read_script: Some(
                r#"
                $memoryCompression = Get-MMAgent | Select-Object -ExpandProperty MemoryCompression
                if ($memoryCompression -eq $false) {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#
                .trim()
                .to_string(),
            ),
            apply_script:
                r#"
                try {
                    Disable-MMAgent -MemoryCompression
                    Write-Output "Disable RAM Compression Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable RAM Compression Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            undo_script: Some(
                r#"
                try {
                    Enable-MMAgent -MemoryCompression
                    Write-Output "Disable RAM Compression Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable RAM Compression Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true,

    )
}

pub fn disable_local_firewall() -> Tweak {
    Tweak::powershell_tweak(
        "Disable Local Firewall".to_string(),
        "Disables the local Windows Firewall for all profiles by setting the firewall state to `off`. **Warning:** This exposes the system to potential security threats and may cause issues with IPsec server connections.".to_string(),
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
                .trim()
                .to_string(),
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
                .trim()
                .to_string(),
            undo_script: Some(
                r#"
                try {
                    netsh advfirewall set allprofiles state on
                    Write-Output "Disable Local Firewall Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable Local Firewall Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true,

    )
}

pub fn disable_success_auditing() -> Tweak {
    Tweak::powershell_tweak(
        "Disable Success Auditing".to_string(),
        "Disables auditing of successful events across all categories, reducing the volume of event logs and system overhead. Security events in the Windows Security log are not affected.".to_string(),
        TweakCategory::Security,
        PowershellTweak {
            id: TweakId::DisableSuccessAuditing,
            read_script: Some(
                r#"
                $auditSettings = (AuditPol /get /category:* /success).Contains("Success Disable")

                if ($auditSettings) {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#
                .trim()
                .to_string(),
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
                .trim()
                .to_string(),

            undo_script: Some(
                r#"
                try {
                    Auditpol /set /category:* /Success:enable
                    Write-Output "Disable Success Auditing Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable Success Auditing: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true,

    )
}

pub fn disable_pagefile() -> Tweak {
    Tweak::powershell_tweak(
        "Disable Pagefile".to_string(),
        "Disables the Windows page file, which is used as virtual memory when physical memory is full. This tweak can improve system performance by reducing disk I/O and preventing paging, but it may cause system instability or application crashes if the system runs out of memory.".to_string(),
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
                "#

                .to_string(),
            ),
            apply_script:"fsutil behavior set encryptpagingfile 0".to_string(),
            undo_script: Some(
               "fsutil behavior set encryptpagingfile 1".to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true,

    )
}

pub fn disable_data_execution_prevention() -> Tweak {
    Tweak::powershell_tweak(
        "Disable Data Execution Prevention".to_string(),
        "Disables Data Execution Prevention (DEP) by setting the `nx` boot configuration option to `AlwaysOff`. This may improve compatibility with older applications but can introduce security risks.".to_string(),
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
                .trim()
                .to_string(),
            ),
            apply_script: "bcdedit.exe /set nx AlwaysOff".to_string(),
            undo_script: Some(
                "bcdedit.exe /set nx OptIn".to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true,
    )
}

pub fn disable_superfetch() -> Tweak {
    Tweak::powershell_tweak(
        "Disable Superfetch".to_string(),
        "Disables the Superfetch service, which preloads frequently used applications into memory to improve performance. This tweak can reduce disk I/O and memory usage but may impact performance in some scenarios.".to_string(),
        TweakCategory::Memory,
        PowershellTweak {
            id: TweakId::DisableSuperfetch,
            read_script: Some(
                r#"
                $superfetchStatus = Get-Service -Name SysMain | Select-Object -ExpandProperty Status

                if ($superfetchStatus -eq "Running") {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#
                .trim()
                .to_string(),
            ),
            apply_script: "sc stop “SysMain” & sc config “SysMain” start=disabled".to_string(),
            undo_script: Some(
                "sc config “SysMain” start=auto & sc start “SysMain”".to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true, // requires reboot
    )
}

/// Function to create the `Low Resolution Mode` Rust tweak.
pub fn low_res_mode() -> Tweak {
    let method = LowResMode::default();

    Tweak::rust_tweak(
        "Low Resolution Mode".to_string(),
        format!(
            "Sets the display to lower resolution and refresh rate to reduce GPU load and improve performance -> {}x{} @{}hz.",
            method.target_state.width, method.target_state.height, method.target_state.refresh_rate
        ),
        TweakCategory::Graphics,
        method,
        TweakWidget::Toggle,
        false,
    )
}

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
        true, // requires reboot
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
        false, // does not require reboot
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
        true, // requires reboot
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
        false, // does not require reboot
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
        true, // requires reboot
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
        true, // requires reboot
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
        "Disable Random Driver Verification".to_string(),
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
        " Paging executive is used to load system files such as kernel and hardware drivers to the page file when needed. Disable will force run into not virtual memory".to_string(),
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
        "Disable Prefetcher Service".to_string(),
        "Disables the Prefetcher service by setting the `EnablePrefetcher` registry value to `0`. This may reduce system boot time and improve performance, especially on systems with SSDs.".to_string(),
        TweakCategory::Services,
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
        false, // requires reboot
    )
}

pub fn additional_kernel_worker_threads() -> Tweak {
    Tweak::registry_tweak(
        "Additional Worker Threads".to_string(),
        "Increases the number of kernel worker threads by setting the AdditionalCriticalWorkerThreads and AdditionalDelayedWorkerThreads values to match the number of logical processors in the system. This tweak boosts performance in multi-threaded workloads by allowing the kernel to handle more concurrent operations, improving responsiveness and reducing bottlenecks in I/O-heavy or CPU-bound tasks. It ensures that both critical and delayed work items are processed more efficiently, particularly on systems with multiple cores.".to_string(),
        TweakCategory::Kernel,
        RegistryTweak {
            id: TweakId::AdditionalKernelWorkerThreads,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive".to_string(),
                    key: "AdditionalCriticalWorkerThreads".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Executive".to_string(),
                    key: "AdditionalDelayedWorkerThreads".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: None,
                },
            ],
        },
        false,
    )
}

pub fn disable_speculative_execution_mitigations() -> Tweak {
    Tweak::registry_tweak(
        "Disable Speculative Execution Mitigations".to_string(),
        "
Disables speculative execution mitigations by setting the `FeatureSettingsOverride` and `FeatureSettingsOverrideMask` registry values to `3`. This may improve performance but can also introduce security risks.
".trim().to_string(),
        TweakCategory::Security,
        RegistryTweak {
            id: TweakId::DisableSpeculativeExecutionMitigations,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
                    key: "FeatureSettingsOverride".to_string(),
                    target_value: RegistryKeyValue::Dword(3),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
                    key: "FeatureSettingsOverrideMask".to_string(),
                    target_value: RegistryKeyValue::Dword(3),
                    default_value: None,
                },
            ],
        },
        true,
    )
}

pub fn high_performance_visual_settings() -> Tweak {
    Tweak::registry_tweak(
        "High Performance Visual Settings".to_string(),
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
".trim().to_string(),
        TweakCategory::Graphics,
        RegistryTweak {
            id: TweakId::HighPerformanceVisualSettings,
            modifications: vec![
                // 1. Set VisualFXSetting to 'Adjust for best performance' (2)
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects".to_string(),
                    key: "VisualFXSetting".to_string(),
                    target_value: RegistryKeyValue::Dword(2),
                    default_value: Some(RegistryKeyValue::Dword(0)), // Default VisualFXSetting
                },
                // 2. Disable animations when minimizing/maximizing windows
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Control Panel\\Desktop\\WindowMetrics".to_string(),
                    key: "MinAnimate".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(1)),
                },
                // 3. Turn off animated controls and elements inside windows
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Control Panel\\Desktop".to_string(),
                    key: "UserPreferencesMask".to_string(),
                    target_value: RegistryKeyValue::Binary(vec![144, 18, 3, 128, 16, 0, 0, 0]),
                    default_value: Some(RegistryKeyValue::Binary(vec![158, 30, 7, 128, 18, 0, 0, 0])),
                },
                // 4. Disable Aero Peek
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\DWM".to_string(),
                    key: "EnableAeroPeek".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(1)),
                },
                // 5. Turn off live thumbnails for taskbar previews
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced".to_string(),
                    key: "ExtendedUIHoverTime".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(400)), // Default hover time
                },
                // 6. Disable smooth scrolling of list views
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Control Panel\\Desktop".to_string(),
                    key: "SmoothScroll".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(1)),
                },
                // 7. Turn off fading effects for menus and tooltips
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Control Panel\\Desktop".to_string(),
                    key: "UserPreferencesMask".to_string(),
                    target_value: RegistryKeyValue::Binary(vec![144, 18, 3, 128, 16, 0, 0, 0]),
                    default_value: Some(RegistryKeyValue::Binary(vec![158, 30, 7, 128, 18, 0, 0, 0])),
                },
                // 8. Disable font smoothing (ClearType)
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Control Panel\\Desktop".to_string(),
                    key: "FontSmoothing".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(2)),
                },
                // 9. Turn off the shadow effect under mouse pointer
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Control Panel\\Cursors".to_string(),
                    key: "CursorShadow".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(1)),
                },
                // 10. Disable the shadow effect for window borders
                RegistryModification {
                    path: "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize".to_string(),
                    key: "EnableTransparentGlass".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: Some(RegistryKeyValue::Dword(1)),
                },
            ],
        },
        false, // requires reboot
    )
}

pub fn enhanced_kernel_performance() -> Tweak {
    Tweak::registry_tweak(
        "Enhanced Kernel Performance".to_string(),
        "Optimizes various kernel-level settings in the Windows Registry to improve system performance by increasing I/O queue sizes, buffer sizes, and stack sizes, while disabling certain security features. These changes aim to enhance multitasking and I/O operations but may affect system stability and security.".to_string(),
        TweakCategory::Kernel,
        RegistryTweak {
            id: TweakId::EnhancedKernelPerformance,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "MaxDynamicTickDuration".to_string(),
                    target_value: RegistryKeyValue::Dword(10),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "MaximumSharedReadyQueueSize".to_string(),
                    target_value: RegistryKeyValue::Dword(128),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "BufferSize".to_string(),
                    target_value: RegistryKeyValue::Dword(32),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "IoQueueWorkItem".to_string(),
                    target_value: RegistryKeyValue::Dword(32),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "IoQueueWorkItemToNode".to_string(),
                    target_value: RegistryKeyValue::Dword(32),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "IoQueueWorkItemEx".to_string(),
                    target_value: RegistryKeyValue::Dword(32),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "IoQueueThreadIrp".to_string(),
                    target_value: RegistryKeyValue::Dword(32),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "ExTryQueueWorkItem".to_string(),
                    target_value: RegistryKeyValue::Dword(32),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "ExQueueWorkItem".to_string(),
                    target_value: RegistryKeyValue::Dword(32),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "IoEnqueueIrp".to_string(),
                    target_value: RegistryKeyValue::Dword(32),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "XMMIZeroingEnable".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "UseNormalStack".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "UseNewEaBuffering".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "StackSubSystemStackSize".to_string(),
                    target_value: RegistryKeyValue::Dword(65536),
                    default_value: None,
                },
            ],
        },
        false, // does not require reboot
    )
}

pub fn split_large_caches() -> Tweak {
    Tweak::registry_tweak(
        "Split Large Caches".to_string(),
        "This registry key is used to enable the splitting of large caches in the Windows operating system. This setting can help improve system performance by optimizing how the kernel handles large cache sizes, particularly in systems with significant memory resources.".to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::SplitLargeCaches,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "SplitLargeCaches".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: Some(RegistryKeyValue::Dword(0)),
                },
            ],
        },
        true, // requires reboot
    )
}

pub fn se_lock_memory_privilege() -> Tweak {
    Tweak::group_policy_tweak(
        "SeLockMemoryPrivilege".to_string(),
        "The SeLockMemoryPrivilege group policy setting allows a process to lock pages in physical memory, preventing them from being paged out to disk. This can improve performance for applications that require fast, consistent access to critical data by keeping it always available in RAM.".to_string(),
        TweakCategory::Memory,
        GroupPolicyTweak {
            id: TweakId::SeLockMemoryPrivilege,
            key: "SeLockMemoryPrivilege".to_string(),
            value: GroupPolicyValue::Enabled,
        },
        true,
    )
}

pub fn disable_protected_services() -> Tweak {
    Tweak::registry_tweak(
        "Disable Protected Services".to_string(),
        "Disables multiple services which can only be stopped by modifying the registry. These will not break your system, but will stop networking functionality.".to_string(),
        TweakCategory::Services,
        RegistryTweak {
            id: TweakId::DisableProtectedServices,
            modifications: vec![
                RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\DoSvc".to_string(),
                key: "Start".to_string(),
                target_value: RegistryKeyValue::Dword(4),
                default_value: Some(RegistryKeyValue::Dword(3)),
            },
            RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\Dhcp".to_string(),
                key: "Start".to_string(),
                target_value: RegistryKeyValue::Dword(4),
                default_value: Some(RegistryKeyValue::Dword(2)),
            },
            RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\NcbService"
                    .to_string(),
                key: "Start".to_string(),
                target_value: RegistryKeyValue::Dword(4),
                default_value: Some(RegistryKeyValue::Dword(2)),
            },
            RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\netprofm"
                    .to_string(),
                key: "Start".to_string(),
                target_value: RegistryKeyValue::Dword(4),
                default_value: Some(RegistryKeyValue::Dword(2)),
            },
            RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\nsi".to_string(),
                key: "Start".to_string(),
                target_value: RegistryKeyValue::Dword(4),
                default_value: Some(RegistryKeyValue::Dword(2)),
            },
            RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\RmSvc".to_string(),
                key: "Start".to_string(),
                target_value: RegistryKeyValue::Dword(4),
                default_value: Some(RegistryKeyValue::Dword(2)),
            }
            ],
        },
        true, // requires reboot
    )
}

pub fn disable_security_accounts_manager() -> Tweak {
    Tweak::registry_tweak(
        "Disable Security Accounts Manager".to_string(),
        "Disables the Security Accounts Manager service by setting the Start registry DWORD to 4."
            .to_string(),
        TweakCategory::Services,
        RegistryTweak {
            id: TweakId::DisableSecurityAccountsManager,
            modifications: vec![RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\SamSs".to_string(),
                key: "Start".to_string(),
                target_value: RegistryKeyValue::Dword(4),
                default_value: Some(RegistryKeyValue::Dword(2)),
            }],
        },
        true, // requires reboot
    )
}

pub fn disable_paging_combining() -> Tweak {
    Tweak::registry_tweak(
        "Disable Paging Combining".to_string(),
        "Disables Windows attempt to save as much RAM as possible, such as sharing pages for images, copy-on-write for data pages, and compression.".to_string(),
        TweakCategory::Memory,
        RegistryTweak {
            id: TweakId::DisablePagingCombining,
            modifications: vec![RegistryModification {
                path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
                key: "DisablePagingCombining".to_string(),
                target_value: RegistryKeyValue::Dword(1),
                default_value: None,
            }],
        },
        true, // requires reboot
    )
}

pub fn aggressive_dpc_handling() -> Tweak {
    Tweak::registry_tweak(
        "Aggressive DPC Handling".to_string(),
        "This tweak modifies kernel-level settings in the Windows Registry to aggressively optimize the handling of Deferred Procedure Calls (DPCs) by disabling timeouts, watchdogs, and minimizing queue depth, aiming to enhance system responsiveness and reduce latency. However, it also removes safeguards that monitor and control long-running DPCs, which could lead to system instability or crashes in certain scenarios, particularly during high-performance or overclocking operations.".to_string(),
        TweakCategory::Kernel,
        RegistryTweak {
            id: TweakId::AggressiveDpcHandling,
            modifications: vec![
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "DpcWatchdogProfileOffset".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "DpcTimeout".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "IdealDpcRate".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "MaximumDpcQueueDepth".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "MinimumDpcRate".to_string(),
                    target_value: RegistryKeyValue::Dword(1),
                    default_value: None,
                },
                RegistryModification {
                    path: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel".to_string(),
                    key: "DpcWatchdogPeriod".to_string(),
                    target_value: RegistryKeyValue::Dword(0),
                    default_value: None,
                },
            ],
        },
        false, // does not require reboot
    )
}
