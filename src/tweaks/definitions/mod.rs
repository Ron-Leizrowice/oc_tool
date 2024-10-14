use disable_processor_idle_states::DisableProcessIdleStates;
use kill_explorer::KillExplorerTweak;
use kill_non_critical_services::KillNonCriticalServicesTweak;
use low_res_mode::LowResMode;
use process_idle_tasks::ProcessIdleTasksTweak;

use super::{powershell::PowershellTweak, Tweak, TweakCategory, TweakId};
use crate::widgets::TweakWidget;

pub mod disable_processor_idle_states;
pub mod kill_explorer;
pub mod kill_non_critical_services;
pub mod low_res_mode;
pub mod process_idle_tasks;
pub mod registry;

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

pub fn aggressive_dpc_handling() -> Tweak {
    Tweak::powershell_tweak(
        "Aggressive DPC Handling".to_string(),
        "This tweak modifies kernel-level settings in the Windows Registry to aggressively optimize the handling of Deferred Procedure Calls (DPCs) by disabling timeouts, watchdogs, and minimizing queue depth, aiming to enhance system responsiveness and reduce latency. However, it also removes safeguards that monitor and control long-running DPCs, which could lead to system instability or crashes in certain scenarios, particularly during high-performance or overclocking operations.".to_string(),
        TweakCategory::Kernel,
        PowershellTweak {
            id: TweakId::AggressiveDpcHandling,
            read_script: Some(
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"
                try {
                    $offset = (Get-ItemProperty -Path $path -Name DpcWatchdogProfileOffset -ErrorAction SilentlyContinue).DpcWatchdogProfileOffset
                    $timeout = (Get-ItemProperty -Path $path -Name DpcTimeout -ErrorAction SilentlyContinue).DpcTimeout
                    $idealRate = (Get-ItemProperty -Path $path -Name IdealDpcRate -ErrorAction SilentlyContinue).IdealDpcRate
                    $maxQueue = (Get-ItemProperty -Path $path -Name MaximumDpcQueueDepth -ErrorAction SilentlyContinue).MaximumDpcQueueDepth
                    $minRate = (Get-ItemProperty -Path $path -Name MinimumDpcRate -ErrorAction SilentlyContinue).MinimumDpcRate
                    $period = (Get-ItemProperty -Path $path -Name DpcWatchdogPeriod -ErrorAction SilentlyContinue).DpcWatchdogPeriod

                    if ($offset -eq 0 -and `
                        $timeout -eq 0 -and `
                        $idealRate -eq 1 -and `
                        $maxQueue -eq 1 -and `
                        $minRate -eq 1 -and `
                        $period -eq 0) {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                } catch {
                    Write-Error "Failed to read one or more registry values."
                }
                "#
                .trim()
                .to_string(),
            ),
            apply_script: r#"
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcWatchdogProfileOffset -Value 0
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcTimeout -Value 0
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name IdealDpcRate -Value 1
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name MaximumDpcQueueDepth -Value 1
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name MinimumDpcRate -Value 1
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcWatchdogPeriod -Value 0
                "#
                .trim()
                .to_string(),
            undo_script: Some(
                r#"
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcWatchdogProfileOffset -Value 10000
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcTimeout -ErrorAction SilentlyContinue
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name IdealDpcRate -ErrorAction SilentlyContinue
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name MaximumDpcQueueDepth -ErrorAction SilentlyContinue
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name MinimumDpcRate -ErrorAction SilentlyContinue
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcWatchdogPeriod -ErrorAction SilentlyContinue
                "#
                .trim()
                .to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        false,

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
