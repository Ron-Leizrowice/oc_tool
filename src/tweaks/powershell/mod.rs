// src/tweaks/powershell/mod.rs

pub mod method;

use method::PowershellTweak;

use super::{Tweak, TweakCategory};
use crate::tweaks::TweakId;

pub fn all_powershell_tweaks<'a>() -> Vec<(TweakId, Tweak<'a>)> {
    vec![
        (TweakId::ProcessIdleTasks, process_idle_tasks()),
        (TweakId::DisableHPET, disable_hpet()),
        (TweakId::DisableRamCompression, disable_ram_compression()),
        (TweakId::DisableLocalFirewall, disable_local_firewall()),
        (TweakId::DisableSuccessAuditing, disable_success_auditing()),
        (TweakId::DisablePagefile, disable_pagefile()),
        (
            TweakId::DisableDataExecutionPrevention,
            disable_data_execution_prevention(),
        ),
        (TweakId::DisableSuperfetch, disable_superfetch()),
    ]
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
