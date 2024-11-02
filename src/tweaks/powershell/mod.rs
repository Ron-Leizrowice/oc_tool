// src/tweaks/powershell/mod.rs

pub mod method;

use std::collections::HashMap;

use method::{PowershellAction, PowershellTweak};

use super::{Tweak, TweakCategory, TweakOption};
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
        (TweakId::DisableCpuIdleStates, disable_cpu_idle_states()),
        (TweakId::DisableDps, disable_dps()),
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
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Run,
                    &PowershellAction {
                        script:"rundll32.exe advapi32.dll,ProcessIdleTasks",
            state: None})],
            ),

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
            read_script: Some(
                r#"bcdedit /enum | Select-String "useplatformclock" | ForEach-Object {
                    if (($_ -split '\s+')[1] -eq 'Yes') {
                        'True'
                    } else {
                        'False'
                    }
                } | Select-Object -Unique"#
            ),
            options: HashMap::from_iter(
                vec![
                    (
                        TweakOption::Enabled(false),
                        &PowershellAction {
                            script: r#"
                                bcdedit /deletevalue useplatformclock
                                bcdedit /set disabledynamictick yes
                            "#,
                            state: Some("True"),
                        }
                    ),
                    (
                        TweakOption::Enabled(true),
                        &PowershellAction {
                            script: r#"
                                bcdedit /set useplatformclock true
                                bcdedit /set disabledynamictick no
                            "#,
                            state: Some("False"),
                        },
                    ),
                ],
            ),
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
                $ramCompression = Get-MMAgent | Select-Object -ExpandProperty MemoryCompression

                if ($ramCompression -eq $true) {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#.trim(),
            ),
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Enabled(true),
                    &PowershellAction {
                        script: r#"Disable-MMAgent -mc"#,
                        state: Some("True"),
                    }),
                    (TweakOption::Enabled(false),
                    &PowershellAction {
                        script: r#"Enable-MMAgent -mc"#,
                        state: Some("False"),
                    }),
                ],
            ),
        },
        true,
    )
}

pub fn disable_local_firewall<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable Local Firewall",
        "Disables the local Windows Firewall for all profiles by setting the firewall state to `off`.",
        TweakCategory::Security,
        PowershellTweak {
            id: TweakId::DisableLocalFirewall,
            read_script: Some(
                r#"
                $firewallState = netsh advfirewall show allprofiles state | Select-String "State" | ForEach-Object { $_.Line }

                if ($firewallState -match "off") {
                    "Off"
                } else {
                    "On"
                }
                "#
                .trim(),
            ),
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Enabled(false),
                    &PowershellAction {
                        script: "netsh advfirewall set allprofiles state on",
                        state: Some("On"),
                    }),
                    (TweakOption::Enabled(true),
                    &PowershellAction {
                        script: "netsh advfirewall set allprofiles state off",
                        state: Some("Off"),
                    }),
                ],
            ),
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
                $auditSettings = AuditPol /get /category:* | Where-Object { $_ -match "No Auditing" }

                if ($auditSettings.Count -gt 0) {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#.trim(),
            ),
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Enabled(false),
                    &PowershellAction {
                        script: "Auditpol /set /category:* /Success:enable",
                        state: Some("Disabled"),
                    }),
                    (TweakOption::Enabled(true),
                    &PowershellAction {
                        script: "Auditpol /set /category:* /Success:disable",
                        state: Some("Enabled"),
                    }),
                ],
            ),
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
                    "Disabled"
                } else {
                    "Enabled"
                }
                "#.trim(),
            ),
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Enabled(false), // Keep pagefile enabled
                    &PowershellAction {
                        script: "wmic computersystem set AutomaticManagedPagefile=True",
                        state: Some("Enabled"),  // Changed to match actual state
                    }),
                    (TweakOption::Enabled(true), // Disable pagefile
                    &PowershellAction {
                        script: "
wmic computersystem set AutomaticManagedPagefile=False
wmic pagefileset delete
",
                        state: Some("Disabled"),  // Changed to match actual state
                    }),
                ],
            ),
        },
        true, // requires reboot
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
                    "Disabled"
                } else {
                    "Enabled"
                }
                "#
                .trim(),
            ),
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Enabled(false),
                    &PowershellAction {
                        script: "bcdedit.exe /set nx OptIn",
                        state: Some("Enabled"),
                    }),
                    (TweakOption::Enabled(true),
                    &PowershellAction {
                        script: "bcdedit.exe /set nx AlwaysOff",
                        state: Some("Disabled"),
                    }),
                ],
            ),
        },
        true, // requires reboot
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
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#
                .trim(),
            ),
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Enabled(false),
                    &PowershellAction {
                        script: r#"Set-Service -Name "SysMain" -StartupType Automatic -Status Running"#,
                        state: Some("Disabled"),
                    }),
                    (TweakOption::Enabled(true),
                    &PowershellAction {
                        script: r#"Stop-Service -Force -Name "SysMain"; Set-Service -Name "SysMain" -StartupType Disabled"#,
                        state: Some("Enabled"),
                    }),

                ],
            ),
        },
        true, // requires reboot
    )
}

fn disable_cpu_idle_states<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable CPU Idle States",
        "Disables CPU idle states (C-states) to prevent the processor from entering low-power states. This tweak can reduce latency and improve performance in real-time applications but may increase power consumption and heat output.",
        TweakCategory::Power,
        PowershellTweak {
            id: TweakId::DisableCpuIdleStates,
            read_script: Some(
                r#"
powercfg /QH |
    Select-String -Pattern 'GUID Alias:\s+IDLEDISABLE' -Context 0,6 |
    ForEach-Object {
        $_.Context.PostContext |
        Select-String 'Current AC Power Setting Index:' |
        ForEach-Object {
            ($_ -split ':')[1].Trim()
        }
    }
                "#.trim(),
            ),
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Enabled(false),
                    &PowershellAction {
                        script: r#"
                        powercfg.exe /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR IdleDisable 0
                        powercfg.exe /setactive SCHEME_CURRENT
                        "#,
                        state: Some("0x00000000"),
                    }),
                    (TweakOption::Enabled(true),
                    &PowershellAction {
                        script: r#"
                        powercfg.exe /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR IdleDisable 1
                        powercfg.exe /setactive SCHEME_CURRENT
                        "#,
                        state: Some("0x00000001"),
                    }),
                ],
            ),
        },
        false, // does not require reboot
    )
}

fn disable_dps<'a>() -> Tweak<'a> {
    Tweak::powershell_tweak(
        "Disable Diagnostic Policy Service",
        "Disables the Diagnostic Policy Service (DPS), which is responsible for troubleshooting and diagnostics in Windows.",
        TweakCategory::System,
        PowershellTweak {
            id: TweakId::DisableDps,
            read_script: Some(r#"(Get-Service -Name "DPS").Status"#),
            options: HashMap::from_iter(
                vec![
                    (TweakOption::Enabled(false),
                    &PowershellAction {
                        script: r#"Start-Service -Force -Name "DPS""#,
                        state: Some("Running"),
                    }),
                    (TweakOption::Enabled(true),
                    &PowershellAction {
                        script: r#"Stop-Service -Name "DPS" -Force"#,
                        state: Some("Stopped"),
                    }),
                ],
            ),
        },
        false, // does not require reboot
    )
}