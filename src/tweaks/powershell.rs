// src/tweaks/powershell.rs

use std::process::Command;

use tracing::{debug, error, info, warn};

use super::{Tweak, TweakCategory, TweakId, TweakMethod};

/// Represents a PowerShell-based tweak, including scripts to read, apply, and undo the tweak.
#[derive(Clone, Debug)]
pub struct PowershellTweak {
    /// The unique ID of the tweak
    pub id: TweakId,
    /// PowerShell script to read the current state of the tweak.
    pub read_script: Option<String>,
    /// PowerShell script to apply the tweak.
    pub apply_script: String,
    /// PowerShell script to undo the tweak.
    pub undo_script: Option<String>,
    /// The target state of the tweak (e.g., the expected output of the read script when the tweak is enabled).
    pub target_state: Option<String>,
}

impl PowershellTweak {
    /// Reads the current state of the tweak by executing the `read_script`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` with the current state if `read_script` is defined and succeeds.
    /// - `Ok(None)` if no `read_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn read_current_state(&self) -> Result<Option<String>, anyhow::Error> {
        if let Some(script) = &self.read_script {
            info!(
                "{:?} -> Reading current state of PowerShell tweak.",
                self.id
            );
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    script,
                ])
                .output()
                .map_err(|e| {
                    anyhow::Error::msg(format!(
                        "{:?} -> Failed to execute PowerShell script '{}': {:?}",
                        self.id, script, e
                    ))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(
                    "{:?} -> PowerShell script '{}' failed with error: {}",
                    self.id,
                    script,
                    stderr.trim()
                );
                return Err(anyhow::Error::msg(format!(
                    "PowerShell script '{}' failed with error: {}",
                    script,
                    stderr.trim()
                )));
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!(
                "{:?} -> PowerShell script output: {}",
                self.id,
                stdout.trim()
            );
            Ok(Some(stdout.trim().to_string()))
        } else {
            debug!(
                "{:?} -> No read script defined for PowerShell tweak. Skipping read operation.",
                self.id
            );
            Ok(None)
        }
    }
}

impl TweakMethod for PowershellTweak {
    /// Checks if the tweak is currently enabled by comparing the current value to the default value.
    /// If the current value matches the default value, the tweak is considered enabled.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation succeeds and the tweak is enabled.
    /// - `Ok(false)` if the operation succeeds and the tweak is disabled.
    /// - `Err(anyhow::Error)` if the operation fails.
    fn initial_state(&self) -> Result<bool, anyhow::Error> {
        if let Some(target_state) = &self.target_state {
            info!("{:?} -> Checking if PowerShell tweak is enabled.", self.id);
            match self.read_current_state() {
                Ok(Some(current_state)) => {
                    // check if the target state string is contained in the current state
                    let is_enabled = current_state.contains(target_state);
                    debug!(
                        "{:?} -> Current state: '{}', Target state: '{}', Enabled: {}",
                        self.id, current_state, target_state, is_enabled
                    );
                    Ok(is_enabled)
                }
                Ok(None) => {
                    warn!(
                        "{:?} -> No read script defined for PowerShell tweak. Assuming disabled.",
                        self.id
                    );
                    Ok(false)
                }
                Err(e) => {
                    error!(
                        error = ?e,
                        "{:?} -> Failed to read current state of PowerShell tweak.", self.id
                    );
                    Err(e)
                }
            }
        } else {
            warn!(
                "{:?} -> No target state defined for PowerShell tweak. Assuming disabled.",
                self.id
            );
            Ok(false)
        }
    }

    /// Executes the `apply_script` to apply the tweak synchronously.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn apply(&self) -> Result<(), anyhow::Error> {
        info!(
            "{:?} -> Applying PowerShell tweak using script '{}'.",
            self.id, &self.apply_script
        );

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &self.apply_script,
            ])
            .output()
            .map_err(|e| {
                anyhow::Error::msg(format!(
                    "{:?} -> Failed to execute PowerShell script '{}': {:?}",
                    self.id, &self.apply_script, e
                ))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            debug!(
                "{:?} -> Apply script executed successfully. Output: {}",
                self.id,
                stdout.trim()
            );
            Ok(())
        } else {
            error!(
                "{:?} -> PowerShell script '{}' failed with error: {}",
                self.id,
                &self.apply_script,
                stderr.trim()
            );
            Err(anyhow::Error::msg(format!(
                "PowerShell script '{}' failed with error: {}",
                &self.apply_script,
                stderr.trim()
            )))
        }
    }

    /// Executes the `undo_script` to revert the tweak synchronously.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully or no `undo_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn revert(&self) -> Result<(), anyhow::Error> {
        if let Some(script) = &self.undo_script {
            info!(
                "{:?} -> Reverting PowerShell tweak using script '{}'.",
                self.id, script
            );

            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    script,
                ])
                .output()
                .map_err(|e| {
                    anyhow::Error::msg(format!(
                        "{:?} -> Failed to execute PowerShell script '{}': {:?}",
                        self.id, script, e
                    ))
                })?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                debug!(
                    "{:?} -> Revert script executed successfully. Output: {}",
                    self.id,
                    stdout.trim()
                );
                Ok(())
            } else {
                error!(
                    "{:?} -> PowerShell script '{}' failed with error: {}",
                    self.id,
                    script,
                    stderr.trim()
                );
                Err(anyhow::Error::msg(format!(
                    "PowerShell script '{}' failed with error: {}",
                    script,
                    stderr.trim()
                )))
            }
        } else {
            warn!(
                "{:?} -> No undo script defined for PowerShell tweak. Skipping revert operation.",
                self.id
            );
            Ok(())
        }
    }
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

pub fn additional_kernel_worker_threads() -> Tweak {
    Tweak::powershell_tweak(
        "Additional Worker Threads".to_string(),
        "Increases the number of kernel worker threads by setting the AdditionalCriticalWorkerThreads and AdditionalDelayedWorkerThreads values to match the number of logical processors in the system. This tweak boosts performance in multi-threaded workloads by allowing the kernel to handle more concurrent operations, improving responsiveness and reducing bottlenecks in I/O-heavy or CPU-bound tasks. It ensures that both critical and delayed work items are processed more efficiently, particularly on systems with multiple cores.".to_string(),
        TweakCategory::Kernel,
        PowershellTweak {
            id: TweakId::AdditionalKernelWorkerThreads,
            read_script: Some(
                r#"
                Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalCriticalWorkerThreads
                Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalDelayedWorkerThreads
                "#
                .trim()
                .to_string(),
            ),
            apply_script:
                r#"
                $additionalThreads = (Get-WmiObject -Class Win32_Processor).NumberOfLogicalProcessors
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalCriticalWorkerThreads -Value $additionalThreads
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalDelayedWorkerThreads -Value $additionalThreads
                "#
                .trim()
                .to_string()
            ,
            undo_script: Some(
                r#"
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalCriticalWorkerThreads
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalDelayedWorkerThreads
                "#
                .trim()
                .to_string(),
            ),
            target_state: None,
        },
        false,

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

pub fn enhanced_kernel_performance() -> Tweak {
    Tweak::powershell_tweak(
        "Enhanced Kernel Performance".to_string(),
        "Optimizes various kernel-level settings in the Windows Registry to improve system performance by increasing I/O queue sizes, buffer sizes, and stack sizes, while disabling certain security features. These changes aim to enhance multitasking and I/O operations but may affect system stability and security.".to_string(),
        TweakCategory::Kernel,
        PowershellTweak {
            id: TweakId::EnhancedKernelPerformance,
            read_script: Some(
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"
                try {
                    $maxDynamicTickDuration = (Get-ItemProperty -Path $path -Name "MaxDynamicTickDuration" -ErrorAction SilentlyContinue).MaxDynamicTickDuration
                    $maxSharedReadyQueueSize = (Get-ItemProperty -Path $path -Name "MaximumSharedReadyQueueSize" -ErrorAction SilentlyContinue).MaximumSharedReadyQueueSize
                    $bufferSize = (Get-ItemProperty -Path $path -Name "BufferSize" -ErrorAction SilentlyContinue).BufferSize
                    $ioQueueWorkItem = (Get-ItemProperty -Path $path -Name "IoQueueWorkItem" -ErrorAction SilentlyContinue).IoQueueWorkItem
                    $ioQueueWorkItemToNode = (Get-ItemProperty -Path $path -Name "IoQueueWorkItemToNode" -ErrorAction SilentlyContinue).IoQueueWorkItemToNode
                    $ioQueueWorkItemEx = (Get-ItemProperty -Path $path -Name "IoQueueWorkItemEx" -ErrorAction SilentlyContinue).IoQueueWorkItemEx
                    $ioQueueThreadIrp = (Get-ItemProperty -Path $path -Name "IoQueueThreadIrp" -ErrorAction SilentlyContinue).IoQueueThreadIrp
                    $exTryQueueWorkItem = (Get-ItemProperty -Path $path -Name "ExTryQueueWorkItem" -ErrorAction SilentlyContinue).ExTryQueueWorkItem
                    $exQueueWorkItem = (Get-ItemProperty -Path $path -Name "ExQueueWorkItem" -ErrorAction SilentlyContinue).ExQueueWorkItem
                    $ioEnqueueIrp = (Get-ItemProperty -Path $path -Name "IoEnqueueIrp" -ErrorAction SilentlyContinue).IoEnqueueIrp
                    $xMMIZeroingEnable = (Get-ItemProperty -Path $path -Name "XMMIZeroingEnable" -ErrorAction SilentlyContinue).XMMIZeroingEnable
                    $useNormalStack = (Get-ItemProperty -Path $path -Name "UseNormalStack" -ErrorAction SilentlyContinue).UseNormalStack
                    $useNewEaBuffering = (Get-ItemProperty -Path $path -Name "UseNewEaBuffering" -ErrorAction SilentlyContinue).UseNewEaBuffering
                    $stackSubSystemStackSize = (Get-ItemProperty -Path $path -Name "StackSubSystemStackSize" -ErrorAction SilentlyContinue).StackSubSystemStackSize

                    if (
                        $maxDynamicTickDuration -eq 10 -and
                        $maxSharedReadyQueueSize -eq 128 -and
                        $bufferSize -eq 32 -and
                        $ioQueueWorkItem -eq 32 -and
                        $ioQueueWorkItemToNode -eq 32 -and
                        $ioQueueWorkItemEx -eq 32 -and
                        $ioQueueThreadIrp -eq 32 -and
                        $exTryQueueWorkItem -eq 32 -and
                        $exQueueWorkItem -eq 32 -and
                        $ioEnqueueIrp -eq 32 -and
                        $xMMIZeroingEnable -eq 0 -and
                        $useNormalStack -eq 1 -and
                        $useNewEaBuffering -eq 1 -and
                        $stackSubSystemStackSize -eq 65536
                    ) {
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
            apply_script:
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"
                try {
                    Set-ItemProperty -Path $path -Name "MaxDynamicTickDuration" -Value 10 -Type DWord
                    Set-ItemProperty -Path $path -Name "MaximumSharedReadyQueueSize" -Value 128 -Type DWord
                    Set-ItemProperty -Path $path -Name "BufferSize" -Value 32 -Type DWord
                    Set-ItemProperty -Path $path -Name "IoQueueWorkItem" -Value 32 -Type DWord
                    Set-ItemProperty -Path $path -Name "IoQueueWorkItemToNode" -Value 32 -Type DWord
                    Set-ItemProperty -Path $path -Name "IoQueueWorkItemEx" -Value 32 -Type DWord
                    Set-ItemProperty -Path $path -Name "IoQueueThreadIrp" -Value 32 -Type DWord
                    Set-ItemProperty -Path $path -Name "ExTryQueueWorkItem" -Value 32 -Type DWord
                    Set-ItemProperty -Path $path -Name "ExQueueWorkItem" -Value 32 -Type DWord
                    Set-ItemProperty -Path $path -Name "IoEnqueueIrp" -Value 32 -Type DWord
                    Set-ItemProperty -Path $path -Name "XMMIZeroingEnable" -Value 0 -Type DWord
                    Set-ItemProperty -Path $path -Name "UseNormalStack" -Value 1 -Type DWord
                    Set-ItemProperty -Path $path -Name "UseNewEaBuffering" -Value 1 -Type DWord
                    Set-ItemProperty -Path $path -Name "StackSubSystemStackSize" -Value 65536 -Type DWord
                    Write-Output "Enhanced Kernel Performance Tweak Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Enhanced Kernel Performance Tweaks: $_"
                }
                "#
                .trim()
                .to_string()
            ,
            undo_script: Some(
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"
                try {
                    Remove-ItemProperty -Path $path -Name "MaxDynamicTickDuration" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "MaximumSharedReadyQueueSize" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "BufferSize" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "IoQueueWorkItem" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "IoQueueWorkItemToNode" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "IoQueueWorkItemEx" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "IoQueueThreadIrp" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "ExTryQueueWorkItem" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "ExQueueWorkItem" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "IoEnqueueIrp" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "XMMIZeroingEnable" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "UseNormalStack" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "UseNewEaBuffering" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path $path -Name "StackSubSystemStackSize" -ErrorAction SilentlyContinue
                    Write-Output "Enhanced Kernel Performance Tweaks Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Enhanced Kernel Performance Tweaks: $_"
                }
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

pub fn disable_speculative_execution_mitigations() -> Tweak {
    Tweak::powershell_tweak(
        "Disable Speculative Execution Mitigations".to_string(),
        "Disables speculative execution mitigations by setting the `FeatureSettingsOverride` and `FeatureSettingsOverrideMask` registry values to `3`. This may improve performance but can also introduce security risks.".to_string(),
        TweakCategory::Security,
        PowershellTweak {
            id: TweakId::DisableSpeculativeExecutionMitigations,
            read_script: Some(
                r#"
                $featureSettingsOverride = (Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name FeatureSettingsOverride -ErrorAction SilentlyContinue).FeatureSettingsOverride
                $featureSettingsOverrideMask = (Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name FeatureSettingsOverrideMask -ErrorAction SilentlyContinue).FeatureSettingsOverrideMask

                if ($featureSettingsOverride -eq 3 -and $featureSettingsOverrideMask -eq 3) {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#
                .trim()
                .to_string(),
            ),
            apply_script: r#"
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name FeatureSettingsOverride -Value 3
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name FeatureSettingsOverrideMask -Value 3
                "#
                .trim()
                .to_string(),
            undo_script: Some(
                r#"
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name FeatureSettingsOverride -ErrorAction SilentlyContinue
                Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name FeatureSettingsOverrideMask -ErrorAction SilentlyContinue
                "#
                .trim()
                .to_string(),
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
            apply_script: "bcdedit /set {current} nx AlwaysOff".to_string(),
            undo_script: Some(
                "bcdedit /set {current} nx OptIn".to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true,
    )
}

pub fn disable_process_idle_states() -> Tweak {
    Tweak::powershell_tweak(
        "Disable Process Idle States".to_string(),
        "Disables processor idle states (C-states) to prevent the CPU from entering low-power states during idle periods. This tweak can improve system responsiveness but may increase power consumption and heat output.".to_string(),
        TweakCategory::Power,
        PowershellTweak {
            id: TweakId::DisableProcessIdleStates,
            read_script: Some(
                r#"
                # Run powercfg command and store the output
                $output = powercfg /qh scheme_current sub_processor

                # Use regex to find the block containing "Processor idle disable"
                $idleDisableBlock = $output | Select-String -Pattern "Power Setting GUID: 5d76a2ca-e8c0-402f-a133-2158492d58ad\s+\(Processor idle disable\)" -Context 0,7

                if ($idleDisableBlock) {
                    # Find the line containing "Current AC Power Setting Index:"
                    $acSettingLine = $idleDisableBlock.Context.PostContext | Select-String -Pattern "Current AC Power Setting Index:" | Select-Object -First 1

                    if ($acSettingLine) {
                        # Extract and output only the hexadecimal value
                        $acSettingValue = $acSettingLine -replace ".*:\s*(0x[0-9A-Fa-f]+).*", '$1'
                        Write-Output $acSettingValue
                    } else {
                        throw "Error: AC Power Setting Index not found in the Processor idle disable block"
                    }
                } else {
                    throw "Error: Processor idle disable setting not found"
                }
                "#
                .trim()
                .to_string(),
            ),
            apply_script: r#"
                powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR IdleDisable 1
                powercfg /setactive SCHEME_CURRENT
                "#.to_string(),
            undo_script: Some(
                r#"
                powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR IdleDisable 0
                powercfg /setactive SCHEME_CURRENT
                "#.to_string(),
            ),
            target_state: Some("0x00000001".to_string()),
        },
        false,
    )
}

pub fn high_performance_visual_settings() -> Tweak {
    Tweak::powershell_tweak(
        "High Performance Visual Settings".to_string(),
        "This tweak adjusts Windows visual settings to prioritize performance over appearance, including drastic changes to display settings. Here's what it does:

1. Sets the overall Visual Effects setting to 'Adjust for best performance'
2. Disables transparency effects in the taskbar, Start menu, and Action Center
3. Disables animations when minimizing and maximizing windows
4. Turns off animated controls and elements inside windows
5. Disables Aero Peek (the feature that shows desktop previews when hovering over the Show Desktop button)
6. Turns off live thumbnails for taskbar previews
7. Disables smooth scrolling of list views
8. Turns off fading effects for menus and tooltips
9. Disables font smoothing (ClearType)
10. Turns off the shadow effect under mouse pointer
11. Disables the shadow effect for window borders
12. Sets the display resolution to 800x600
13. Lowers the refresh rate to 30Hz".to_string(),
        TweakCategory::Graphics,
        PowershellTweak {
            id: TweakId::HighPerformanceVisualSettings,
            read_script: Some(r#"
                $path = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects"
                $visualFxSetting = Get-ItemProperty -Path $path -Name VisualFXSetting -ErrorAction SilentlyContinue

                $currentResolution = Get-WmiObject -Class Win32_VideoController | Select-Object -ExpandProperty VideoModeDescription
                $currentRefreshRate = Get-WmiObject -Class Win32_VideoController | Select-Object -ExpandProperty CurrentRefreshRate

                $status = @{
                    HighPerformanceMode = $false
                    Resolution = $currentResolution
                    RefreshRate = $currentRefreshRate
                }

                if ($visualFxSetting -and $visualFxSetting.VisualFXSetting -eq 2) {
                    $status.HighPerformanceMode = $true
                }

                return $status | ConvertTo-Json
            "#.to_string()),
            apply_script: r#"
                # Set visual effects to best performance
                Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects" -Name VisualFXSetting -Value 2

                # Disable individual visual effects
                $path = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"
                Set-ItemProperty -Path $path -Name ListviewAlphaSelect -Value 0
                Set-ItemProperty -Path $path -Name ListviewShadow -Value 0
                Set-ItemProperty -Path $path -Name TaskbarAnimations -Value 0

                $path = "HKCU:\Control Panel\Desktop"
                Set-ItemProperty -Path $path -Name UserPreferencesMask -Value ([byte[]](144,18,3,128,16,0,0,0))
                Set-ItemProperty -Path $path -Name FontSmoothing -Value 0

                $path = "HKCU:\Control Panel\Desktop\WindowMetrics"
                Set-ItemProperty -Path $path -Name MinAnimate -Value 0

                $path = "HKCU:\Software\Microsoft\Windows\DWM"
                Set-ItemProperty -Path $path -Name EnableAeroPeek -Value 0

                # Change display resolution and refresh rate
                $x = 800
                $y = 600
                $refreshRate = 30

                $dll = Add-Type -MemberDefinition @"
                [DllImport("user32.dll")]
                public static extern int ChangeDisplaySettings(ref DEVMODE devMode, int flags);

                [StructLayout(LayoutKind.Sequential)]
                public struct DEVMODE
                {
                    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
                    public string dmDeviceName;
                    public short dmSpecVersion;
                    public short dmDriverVersion;
                    public short dmSize;
                    public short dmDriverExtra;
                    public int dmFields;
                    public int dmPositionX;
                    public int dmPositionY;
                    public int dmDisplayOrientation;
                    public int dmDisplayFixedOutput;
                    public short dmColor;
                    public short dmDuplex;
                    public short dmYResolution;
                    public short dmTTOption;
                    public short dmCollate;
                    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
                    public string dmFormName;
                    public short dmLogPixels;
                    public int dmBitsPerPel;
                    public int dmPelsWidth;
                    public int dmPelsHeight;
                    public int dmDisplayFlags;
                    public int dmDisplayFrequency;
                    public int dmICMMethod;
                    public int dmICMIntent;
                    public int dmMediaType;
                    public int dmDitherType;
                    public int dmReserved1;
                    public int dmReserved2;
                    public int dmPanningWidth;
                    public int dmPanningHeight;
                }
"@ -Name User32 -Namespace Win32 -PassThru

                $dm = New-Object Win32.User32+DEVMODE
                $dm.dmDeviceName = $null
                $dm.dmFormName = $null
                $dm.dmSize = [System.Runtime.InteropServices.Marshal]::SizeOf($dm)
                $dm.dmPelsWidth = $x
                $dm.dmPelsHeight = $y
                $dm.dmDisplayFrequency = $refreshRate
                $dm.dmFields = 0x40000 -bor 0x80000 -bor 0x100000

                $result = [Win32.User32]::ChangeDisplaySettings([ref]$dm, 0)

                if ($result -eq 0) {
                    Write-Output "Display settings changed successfully."
                } else {
                    Write-Output "Failed to change display settings."
                }
            "#.to_string(),
            undo_script: Some(r#"
                # Reset visual effects to system defaults
                Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects" -Name VisualFXSetting -Value 0

                # Reset individual visual effects
                $path = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"
                Set-ItemProperty -Path $path -Name ListviewAlphaSelect -Value 1
                Set-ItemProperty -Path $path -Name ListviewShadow -Value 1
                Set-ItemProperty -Path $path -Name TaskbarAnimations -Value 1

                $path = "HKCU:\Control Panel\Desktop"
                Set-ItemProperty -Path $path -Name UserPreferencesMask -Value ([byte[]](158,30,7,128,18,0,0,0))
                Set-ItemProperty -Path $path -Name FontSmoothing -Value 2

                $path = "HKCU:\Control Panel\Desktop\WindowMetrics"
                Set-ItemProperty -Path $path -Name MinAnimate -Value 1

                $path = "HKCU:\Software\Microsoft\Windows\DWM"
                Set-ItemProperty -Path $path -Name EnableAeroPeek -Value 1

                # Reset display settings to system recommended values
                $dll = Add-Type -MemberDefinition @"
                [DllImport("user32.dll")]
                public static extern int ChangeDisplaySettings(IntPtr devMode, int flags);
"@ -Name User32 -Namespace Win32 -PassThru

                [Win32.User32]::ChangeDisplaySettings([IntPtr]::Zero, 0)
            "#.to_string()),
            target_state: Some(r#"
                {
                    "HighPerformanceMode": true,
                    "Resolution": "800 x 600 x 4294967296 colors",
                    "RefreshRate": 30
                }
            "#.to_string()),
        },
        false,
    )
}
