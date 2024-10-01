// src/tweaks/powershell_tweaks.rs

use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use tracing::{debug, error, info, warn};

use super::{method::TweakMethod, Tweak, TweakCategory, TweakId};


/// Represents a PowerShell-based tweak, including scripts to read, apply, and undo the tweak.
#[derive(Clone, Debug)]
pub struct PowershellTweak {
    /// PowerShell script to read the current state of the tweak.
    pub read_script: Option<String>,
    /// PowerShell script to apply the tweak.
    pub apply_script: Option<String>,
    /// PowerShell script to undo the tweak.
    pub undo_script: Option<String>,
    /// The target state of the tweak (e.g., the expected output of the read script when the tweak is enabled).
    pub target_state: Option<String>,
}

impl PowershellTweak {/// Reads the current state of the tweak by executing the `read_script`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` with the current state if `read_script` is defined and succeeds.
    /// - `Ok(None)` if no `read_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    pub fn read_current_state(&self, id: TweakId) -> Result<Option<String>, anyhow::Error> {
        if let Some(script) = &self.read_script {
            info!("{:?} -> Reading current state of PowerShell tweak.", id);
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
                        "{:?} -> Failed to execute PowerShell script '{:?}': {:?}",
                        id, script, e
                    ))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(
                    "{:?} -> PowerShell script '{:?}' failed with error: {:?}",
                    id,
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
            debug!("{:?} -> PowerShell script output: {:?}", id, stdout.trim());
            Ok(Some(stdout.trim().to_string()))
        } else {
            debug!(
                "{:?} -> No read script defined for PowerShell tweak. Skipping read operation.",
                id
            );
            Ok(None)
        }
    }}

impl TweakMethod for PowershellTweak {
    /// Checks if the tweak is currently enabled by comparing the current value to the default value.
    /// If the current value matches the default value, the tweak is considered enabled.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation succeeds and the tweak is enabled.
    /// - `Ok(false)` if the operation succeeds and the tweak is disabled.
    /// - `Err(anyhow::Error)` if the operation fails.
    fn initial_state(&self, id: TweakId) -> Result<bool, anyhow::Error> {
        if let Some(target_state) = &self.target_state {
            info!("{:?} -> Checking if PowerShell tweak is enabled.", id);
            match self.read_current_state(id) {
                Ok(Some(current_state)) => {
                    // check if the target state string is contained in the current state
                    let is_enabled = current_state.contains(target_state);
                    debug!(
                        "{:?} -> Current state: {:?}, Target state: {:?}, Enabled: {:?}",
                        id, current_state, target_state, is_enabled
                    );
                    Ok(is_enabled)
                }
                Ok(None) => {
                    warn!(
                        "{:?} -> No read script defined for PowerShell tweak. Assuming disabled.",
                        id
                    );
                    Ok(false)
                }
                Err(e) => {
                    error!(
                        error = ?e,
                        "{:?} -> Failed to read current state of PowerShell tweak.", id
                    );
                    Err(e)
                }
            }
        } else {
            warn!(
                "{:?} -> No target state defined for PowerShell tweak. Assuming disabled.",
                id
            );
            Ok(false)
        }
    }

    

    /// Executes the `apply_script` to apply the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn apply(&self, id: TweakId) -> Result<(), anyhow::Error> {
        match &self.apply_script {
            Some(script) => {
                info!(
                    "{:?} -> Applying PowerShell tweak using script '{:?}'.",
                    id, script
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
                            "{:?} -> Failed to execute PowerShell script '{:?}': {:?}",
                            id, script, e
                        ))
                    })?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    debug!(
                        "{:?} -> Apply script executed successfully. Output: {:?}",
                        id,
                        stdout.trim()
                    );
                    Ok(())
                } else {
                    error!(
                        "{:?} -> PowerShell script '{}' failed with error: {}",
                        id,
                        script,
                        stderr.trim()
                    );
                    Err(anyhow::Error::msg(format!(
                        "PowerShell script '{}' failed with error: {}",
                        script,
                        stderr.trim()
                    )))
                }
            }
            None => {
                warn!(
                    "{:?} -> No apply script defined for PowerShell tweak. Skipping apply operation.",
                    id
                );
                Ok(())
            }
        }
    }

    /// Executes the `undo_script` to revert the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully or no `undo_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn revert(&self, id: TweakId) -> Result<(), anyhow::Error> {
        if let Some(script) = &self.undo_script {
            info!(
                "{:?} -> Reverting PowerShell tweak using script '{:?}'.",
                id, script
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
                        "{:?} -> Failed to execute PowerShell script '{:?}': {:?}",
                        id, script, e
                    ))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(
                    "{:?} -> PowerShell script '{}' failed with error: {}",
                    id,
                    script,
                    stderr.trim()
                );
                return Err(anyhow::Error::msg(format!(
                    "PowerShell script '{}' failed with error: {}",
                    script,
                    stderr.trim()
                )));
            }

            debug!("{:?} -> Revert script executed successfully.", id);
        } else {
            warn!(
                "{:?} -> No undo script defined for PowerShell tweak. Skipping revert operation.",
                id
            );
        }
        Ok(())
    }
}

pub fn process_idle_tasks() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::ProcessIdleTasks,
        "Process Idle Tasks".to_string(),
        "Forces the execution of scheduled background tasks that are normally run during system idle time. This helps free up system resources by completing these tasks immediately, improving overall system responsiveness and optimizing resource allocation. It can also reduce latency caused by deferred operations in critical system processes.".to_string(),
        TweakCategory::Action,
        PowershellTweak {
            read_script: None,
            apply_script: Some("Rundll32.exe advapi32.dll,ProcessIdleTasks".to_string()),
            undo_script: None,
            target_state: None,
        },
        false, // requires reboot
    )
}

pub fn enable_ultimate_performance_plan() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::UltimatePerformancePlan,
        "Enable Ultimate Performance Plan".to_string(),
        "Activates the Ultimate Performance power plan, which is tailored for demanding workloads by minimizing micro-latencies and boosting hardware performance. It disables power-saving features like core parking, hard disk sleep, and processor throttling, ensuring CPU cores run at maximum frequency. This plan also keeps I/O devices and PCIe links at full power, prioritizing performance over energy efficiency. It’s designed to reduce the delays introduced by energy-saving policies, improving responsiveness in tasks that require consistent, high-throughput system resources..".to_string(),
        TweakCategory::Power,
        PowershellTweak {
            read_script: Some(
                "powercfg /GETACTIVESCHEME".to_string(),
            ),
            apply_script: Some(
                r#"
                powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61
                $ultimatePlans = powercfg /L | Select-String '(Ultimate Performance)' | ForEach-Object { $_.Line }
                $ultimatePlans = @($ultimatePlans | ForEach-Object { $_ -replace 'Power Scheme GUID: ', '' -replace ' \(Ultimate Performance\)', '' -replace '\*$', '' } | ForEach-Object { $_.Trim() })
                for ($i = 0; $i -lt $ultimatePlans.Length - 1; $i++) { powercfg -delete $ultimatePlans[$i] }
                powercfg /SETACTIVE $ultimatePlans[-1]
                "#
                .trim()
                .to_string(),
            ),
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

pub fn additional_kernel_worker_threads() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::AdditionalKernelWorkerThreads,
        "Additional Worker Threads".to_string(),
        "Increases the number of kernel worker threads by setting the AdditionalCriticalWorkerThreads and AdditionalDelayedWorkerThreads values to match the number of logical processors in the system. This tweak boosts performance in multi-threaded workloads by allowing the kernel to handle more concurrent operations, improving responsiveness and reducing bottlenecks in I/O-heavy or CPU-bound tasks. It ensures that both critical and delayed work items are processed more efficiently, particularly on systems with multiple cores.".to_string(),
        TweakCategory::Kernel,
        PowershellTweak {
            read_script: Some(
                r#"
                Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalCriticalWorkerThreads
                Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalDelayedWorkerThreads
                "#
                .trim()
                .to_string(),
            ),
            apply_script: Some(
                r#"
                $additionalThreads = (Get-WmiObject -Class Win32_Processor).NumberOfLogicalProcessors
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalCriticalWorkerThreads -Value $additionalThreads
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Executive" -Name AdditionalDelayedWorkerThreads -Value $additionalThreads
                "#
                .trim()
                .to_string(),
            ),
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

pub fn disable_hpet() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::DisableHPET,
        "Disable Dynamic Tick".to_string(),
        "Disables the dynamic tick feature, which normally reduces timer interrupts during idle periods to conserve power. By disabling dynamic tick, the system maintains a constant rate of timer interrupts, improving performance in real-time applications by reducing latency and jitter. This tweak is useful in scenarios where consistent, low-latency processing is required, but it may increase power consumption as the CPU will not enter low-power states as frequently.".to_string(),
        TweakCategory::System,
        PowershellTweak {
            read_script: Some(r#"(bcdedit /enum | Select-String "useplatformclock").ToString().Trim()"#.to_string()),
            apply_script: Some(r#"
            bcdedit /deletevalue useplatformclock
            bcdedit /set disabledynamictick yes
            "#.trim().to_string()),
            undo_script: Some(r#"
            bcdedit /set useplatformclock true
            bcdedit /set disabledynamictick no
            "#.trim().to_string()),
            target_state: Some("useplatformclock        Yes".trim().to_string()),
        },
        true,
    )
}

pub fn aggressive_dpc_handling() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::AggressiveDpcHandling,
        "Aggressive DPC Handling".to_string(),
        "This tweak modifies kernel-level settings in the Windows Registry to aggressively optimize the handling of Deferred Procedure Calls (DPCs) by disabling timeouts, watchdogs, and minimizing queue depth, aiming to enhance system responsiveness and reduce latency. However, it also removes safeguards that monitor and control long-running DPCs, which could lead to system instability or crashes in certain scenarios, particularly during high-performance or overclocking operations.".to_string(),
        TweakCategory::Kernel,
        PowershellTweak {
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
            apply_script: Some(
                r#"
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcWatchdogProfileOffset -Value 0
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcTimeout -Value 0
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name IdealDpcRate -Value 1
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name MaximumDpcQueueDepth -Value 1
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name MinimumDpcRate -Value 1
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name DpcWatchdogPeriod -Value 0
                "#
                .trim()
                .to_string(),
            ),
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

pub fn enhanced_kernel_performance() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::EnhancedKernelPerformance,
        "Enhanced Kernel Performance".to_string(),
        "Optimizes various kernel-level settings in the Windows Registry to improve system performance by increasing I/O queue sizes, buffer sizes, and stack sizes, while disabling certain security features. These changes aim to enhance multitasking and I/O operations but may affect system stability and security.".to_string(),
        TweakCategory::Kernel,
        PowershellTweak {
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
            apply_script: Some(
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
                .to_string(),
            ),
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

pub fn disable_ram_compression() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::DisableRamCompression,
        "Disable RAM Compression".to_string(),
        "Disables the RAM compression feature in Windows to potentially improve system performance by reducing CPU overhead. This may lead to higher memory usage.".to_string(),
        TweakCategory::Memory,
        PowershellTweak {
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
            apply_script: Some(
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
            ),
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

pub fn disable_local_firewall() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::DisableLocalFirewall,
        "Disable Local Firewall".to_string(),
        "Disables the local Windows Firewall for all profiles by setting the firewall state to `off`. **Warning:** This exposes the system to potential security threats and may cause issues with IPsec server connections.".to_string(),
        TweakCategory::Security,
        PowershellTweak {
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
            apply_script: Some(
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
            ),
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

pub fn disable_success_auditing() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::DisableSuccessAuditing,
        "Disable Success Auditing".to_string(),
        "Disables auditing of successful events across all categories, reducing the volume of event logs and system overhead. Security events in the Windows Security log are not affected.".to_string(),
        TweakCategory::Security,
        PowershellTweak {
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
            apply_script: Some(
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
            ),
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

pub fn disable_pagefile() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::DisablePagefile,
        "Disable Pagefile".to_string(),
        "Disables the Windows page file, which is used as virtual memory when physical memory is full. This tweak can improve system performance by reducing disk I/O and preventing paging, but it may cause system instability or application crashes if the system runs out of memory.".to_string(),
        TweakCategory::Memory,
        PowershellTweak {
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
            apply_script: Some(
                "fsutil behavior set encryptpagingfile 0".to_string(),
            ),
            undo_script: Some(
               "fsutil behavior set encryptpagingfile 1".to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true,
        
    )
}

pub fn disable_speculative_execution_mitigations() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::DisableSpeculativeExecutionMitigations,
        "Disable Speculative Execution Mitigations".to_string(),
        "Disables speculative execution mitigations by setting the `FeatureSettingsOverride` and `FeatureSettingsOverrideMask` registry values to `3`. This may improve performance but can also introduce security risks.".to_string(),
        TweakCategory::Security,
        PowershellTweak {
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
            apply_script: Some(
                r#"
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name FeatureSettingsOverride -Value 3
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name FeatureSettingsOverrideMask -Value 3
                "#
                .trim()
                .to_string(),
            ),
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

pub fn disable_data_execution_prevention() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::DisableDataExecutionPrevention,
        "Disable Data Execution Prevention".to_string(),
        "Disables Data Execution Prevention (DEP) by setting the `nx` boot configuration option to `AlwaysOff`. This may improve compatibility with older applications but can introduce security risks.".to_string(),
        TweakCategory::Security,
        PowershellTweak {
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
            apply_script: Some(
                "bcdedit /set {current} nx AlwaysOff".to_string(),
            ),
            undo_script: Some(
                "bcdedit /set {current} nx OptIn".to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        },
        true,
        
    )
}

pub fn disable_process_idle_states() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::DisableProcessIdleStates,
        "Disable Process Idle States".to_string(),
        "Disables processor idle states (C-states) to prevent the CPU from entering low-power states during idle periods. This tweak can improve system responsiveness but may increase power consumption and heat output.".to_string(),
        TweakCategory::Power,
        PowershellTweak {
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
            apply_script: Some(
                r#"
                powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR IdleDisable 1
                powercfg /setactive SCHEME_CURRENT
                "#.to_string(),
            ),
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

pub fn kill_all_non_critical_services() -> Arc<Mutex<Tweak>> {
    let services = r#"@(
        "AdobeARMservice",               # Adobe Acrobat Update Service
        "AdobeFlashPlayerUpdateSvc",     # Adobe Flash Player Update Service
        "AdobeUpdateService",            # Adobe Update Service
        "AeLookupSvc",                   # Application Experience
        "AJRouter",                      # AllJoyn Router Service
        "ALG",                           # Application Layer Gateway Service
        "AppIDSvc",                      # Application Identity
        "Appinfo",                       # Application Information
        "AppMgmt",                       # Application Management
        "AppReadiness",                  # App Readiness
        "AppXSvc",                       # AppX Deployment Service
        "AssignedAccessManagerSvc",      # Assigned Access Manager Service
        "AudioEndpointBuilder",          # Windows Audio Endpoint Builder
        "Audiosrv",                      # Windows Audio
        "autotimesvc",                   # Cellular Time
        "AxInstSV",                      # ActiveX Installer
        "BDESVC",                        # BitLocker Drive Encryption Service
        "BluetoothUserService",          # Bluetooth User Support Service
        "BFE",                           # Base Filtering Engine
        "BITS",                          # Background Intelligent Transfer Service
        "BrokerInfrastructure",          # Background Tasks Infrastructure Service
        "Browser",                       # Computer Browser
        "BthAvctpSvc",                   # AVCTP service
        "camsvc",                        # Capability Access Manager Service
        "CaptureService",                # Capability Access Manager Service
        "CertPropSvc",                   # Certificate Propagation
        "ClipSVC",                       # Client License Service
        "CryptSvc",                      # Cryptographic Services
        "defragsvc",                     # Optimize drives
        "DevQueryBroker",                # Device Query Broker
        "DeviceAssociationService",      # Device Association Service
        "DevicesFlowUserSvc",            # Allows ConnectUX and PC Settings to Connect and Pair with WiFi displays and Bluetooth devices.
        "diagnosticshub",                # Microsoft (R) Diagnostics Hub Standard Collector Service
        "DispBrokerDesktopSvc",          # Display Policy Service
        "Dhcp",                          # DHCP Client
        "Dnscache",                      # DNS Client
        "DoSvc",                         # Delivery Optimization
        "DPS",                           # Diagnostic Policy Service
        "DusmSvc",                       # Data Usage
        "EFS",                           # Encrypting File System
        "EntAppSvc",                     # Enterprise App Management Service
        "EventLog",                      # Windows Event Log
        "FrameServer",                   # Windows Camera Frame Server
        "GraphicsPerfSvc",               # GraphicsPerfSvc
        "hidserv",                       # Human Interface Device Service
        "HvHost",                        # Hyper-V Host Compute Service
        "icssvc",                        # Windows Mobile Hotspot Service
        "iphlpsvc",                      # IP Helper
        "lfsvc",                         # Geolocation Service
        "lmhosts",                       # TCP/IP NetBIOS Helper
        "InstallService",                # Microsoft Store Install Service
        "irmon",                         # Infrared monitor service
        "KeyIso",                        # CNG Key Isolation
        "LanmanWorkstation",             # Workstation
        "LanmanServer",                  # Server
        "LicenseManager",                # Windows License Manager Service
        "LxpSvc",                        # Language Experience Service
        "LSM",                           # Local Session Manager
        "MDCoreSvc",                     # Microsoft Defender Core Service
        "mpssvc",                        # Windows Defender Firewall
        "MSDTC",                         # Distributed Transaction Coordinator
        "MSiSCSI",                       # Microsoft iSCSI Initiator Service
        "NaturalAuthentication",         # Natural Authentication
        "NcbService",                    # Network Connection Broker
        "netprofm",                      # Network List Service
        "NgcCtnrSvc",                    # Microsoft Passport Container
        "NgcSvc",                        # Microsoft Passport
        "NPSMSvc",                       # Now Playing Media Service
        "nsi",                           # Network Store Interface Service
        "NVDisplay",                     # NVIDIA Display Driver Service
        "OneSyncSvc",                    # Synchronizes mail, contacts, calendar etc.
        "PcaSvc",                        # Program Compatibility Assistant Service
        "PhoneSvc",                      # Phone Service
        "PimIndexMaintenanceSvc",        # Contact Data
        "pla",                           # Performance Logs & Alerts
        "PlugPlay",                      # Plug and Play
        "PrintNotify",                   # Printer Extensions and Notifications
        "ProfSvc",                       # User Profile Service
        "RasMan",                        # Remote Access Connection Manager
        "RmSvc",                         # Radio Management Service
        "RtkAudioUniversalService",      # Realtek Audio Universal Service
        "SamSs",                         # Security Accounts Manager
        "SCardSvr",                      # Smart Card
        "ScDeviceEnum",                  # Smart Card Device Enumeration Service
        "SCPolicySvc",                   # Smart Card Removal Policy
        "seclogon",                      # Secondary Logon
        "SEMgrSvc",                      # Payments and NFC/SE Manager
        "SensorDataService",             # Sensor Data Service
        "SensorService",                 # Sensor Service
        "SensrSvc",                      # Sensor Monitoring Service
        "SessionEnv",                    # Remote Desktop Configuration
        "Schedule",                      # Task Scheduler
        "ShellHWDetection",              # Shell Hardware Detection
        "shpamsvc",                      # Shared PC Account Manager
        "SmsRouter",                     # Microsoft Windows SMS Router Service
        "smphost",                       # Microsoft Storage Spaces SMP
        "Spooler",                       # Print Spooler
        "sppsvc",                        # Software Protection
        "SstpSvc",                       # Secure Socket Tunneling Protocol Service
        "SSDPSRV",                       # SSDP Discovery
        "StateRepository",               # State Repository Service
        "StiSvc",                        # Windows Image Acquisition
        "StorSvc",                       # Storage Service
        "svsvc",                         # Spot Verifier
        "swprv",                         # Microsoft Software Shadow Copy Provider
        "SysMain",                       # SysMain
        "TabletInputService",            # Touch Keyboard and Handwriting Panel Service
        "tapisrv",                       # Telephony
        "Themes",                        # Themes
        "TermService",                   # Remote Desktop Services
        "TieringEngineService",          # Storage Tiers Management
        "TimeBrokerSvc",                 # Time Broker
        "TokenBroker",                   # Web Account Manager
        "TrkWks",                        # Distributed Link Tracking Client
        "TrustedInstaller",              # Windows Modules Installer
        "UmRdpService",                  # Remote Desktop Services UserMode Port Redirector
        "UnistoreSvc",                   # User Data Storage
        "UserDataSvc",                   # User Data Access
        "UserManager",                   # User Manager
        "UsoSvc",                        # Update Orchestrator Service
        "VaultSvc",                      # Credential Manager
        "vds",                           # Virtual Disk
        "VSS",                           # Volume Shadow Copy
        "WaaSMedicSvc",                  # Windows Update Medic Service
        "WalletService",                 # WalletService
        "WarpJITSvc",                    # WarpJITSvc
        "Wbiosrvc",                      # Windows Biometric Service
        "Wcmsvc",                        # Windows Connection Manager
        "WdiServiceHost",                # Diagnostic Service Host
        "WdiSystemHost",                 # Diagnostic System Host
        "WdNisSvc",                      # Windows Defender Antivirus Network Inspection Service
        "webthreatdefusersvc",           # Web Threat Defense User Service
        "Wecsvc",                        # Windows Event Collector
        "WEPHOSTSVC",                    # Windows Encryption Provider Host Service
        "WerSvc",                        # Windows Error Reporting Service
        "WlanSvc",                       # WLAN AutoConfig
        "wlidsvc",                       # Microsoft Account Sign-in Assistant
        "WiaRpc",                        # Still Image Acquisition Events
        "WinDefend",                     # Windows Defender Antivirus Service
        "WinHttpAutoProxySvc",           # WinHTTP Web Proxy Auto-Discovery Service
        "Winmgmt",                       # Windows Management Instrumentation
        "wmiApSrv",                      # WMI Performance Adapter
        "WpDBusEnum",                    # Portable Device Enumerator Service
        "WpnService",                    # Windows Push Notifications Service
        "WpnUserService",                # Windows Push Notifications User Service
        "wscsvc",                        # Security Center
        "WSearch",                       # Windows Search
        "wuauserv",                      # Windows Update
        "Xbox"                           # All Xbox services
    )"#;

    Tweak::powershell(
        TweakId::KillAllNonCriticalServices,
        "Kill All Non-Critical Services".to_string(),
        "Stops all non-critical services to free up system resources and improve performance. This tweak may cause system instability or data loss.".to_string(),
        TweakCategory::Action,
        PowershellTweak {
            read_script: None,
 
                apply_script: Some(format!(r#"
                    $servicePatterns = {}
    
                    $allServices = Get-Service
                    $failedServices = @()
    
                    foreach ($pattern in $servicePatterns) {{
                        $matchingServices = $allServices | Where-Object {{ $_.Name -like "*$pattern*" -or $_.DisplayName -like "*$pattern*" }}
                        
                        foreach ($service in $matchingServices) {{
                            for ($i = 1; $i -le 5; $i++) {{
                                if ($service.Status -ne 'Stopped') {{
                                    try {{
                                        Stop-Service -InputObject $service -Force -ErrorAction Stop
                                        Write-Output "Stopped service: $($service.Name) ($($service.DisplayName))"
                                        break
                                    }} catch {{
                                        Write-Output "Failed to stop service: $($service.Name) ($($service.DisplayName)). Attempt $i/5."
                                        if ($i -eq 5) {{
                                            $failedServices += $service.Name
                                        }}
                                    }}
                                }} else {{
                                    Write-Output "Service already stopped: $($service.Name) ($($service.DisplayName))"
                                    break
                                }}
                                Start-Sleep -Seconds 2
                            }}
                        }}
                    }}
    
                    if ($failedServices.Count -gt 0) {{
                        Write-Output "The following services failed to be stopped after 5 attempts:"
                        foreach ($failedService in $failedServices) {{
                            Write-Output "- $failedService"
                        }}
                    }} else {{
                        Write-Output "All specified non-critical services have been successfully stopped."
                    }}
                    "#, services)),
            undo_script: None,  target_state: None,
        },
        false,
        
    )
}


pub fn kill_explorer() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::KillExplorer,
        "Kill Explorer".to_string(),
        "Terminates the Windows Explorer process and prevents it from automatically restarting. This can free up system resources but will remove the desktop interface. Use with caution.".to_string(),
        TweakCategory::Action,
        PowershellTweak {
            read_script: Some(r#"
                $explorerProcesses = Get-Process explorer -ErrorAction SilentlyContinue
                $autoRestartValue = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name "AutoRestartShell" -ErrorAction SilentlyContinue

                $status = @{
                    ExplorerRunning = $false
                    AutoRestartEnabled = $false
                }

                if ($explorerProcesses) {
                    $status.ExplorerRunning = $true
                }

                if ($autoRestartValue -and $autoRestartValue.AutoRestartShell -eq 1) {
                    $status.AutoRestartEnabled = $true
                }

                return $status | ConvertTo-Json
            "#.to_string()),
            apply_script: Some(r#"
                Write-Output "Terminating Explorer process and preventing restart..."
                
                # Kill all Explorer processes
                taskkill /F /IM explorer.exe

                # Prevent Explorer from restarting
                New-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name "AutoRestartShell" -Value 0 -PropertyType DWORD -Force

                Write-Output "Explorer has been terminated and prevented from restarting."
            "#.to_string()),
            undo_script: Some(r#"
                Write-Output "Allowing Explorer to restart and starting it..."

                # Allow Explorer to restart automatically
                Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name "AutoRestartShell" -Value 1

                # Start Explorer
                Start-Process explorer.exe

                Write-Output "Explorer has been allowed to restart and has been started."
            "#.to_string()),
            target_state: Some(r#"
                {
                    "ExplorerRunning": false,
                    "AutoRestartEnabled": false
                }
            "#.to_string()),
        },
        false
    )
}


pub fn high_performance_visual_settings() -> Arc<Mutex<Tweak>> {
    Tweak::powershell(
        TweakId::HighPerformanceVisualSettings,
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
            apply_script: Some(r#"
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
            "#.to_string()),
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