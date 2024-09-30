// src/tweaks/powershell_tweaks.rs

use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use tracing::{debug, error, info, warn};

use super::{Tweak, TweakCategory, TweakId, TweakMethod};
use crate::{errors::PowershellError, widgets::TweakWidget};

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

impl PowershellTweak {
    /// Checks if the tweak is currently enabled by comparing the current value to the default value.
    /// If the current value matches the default value, the tweak is considered enabled.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation succeeds and the tweak is enabled.
    /// - `Ok(false)` if the operation succeeds and the tweak is disabled.
    /// - `Err(anyhow::Error)` if the operation fails.
    pub fn is_powershell_script_enabled(&self, id: TweakId) -> Result<bool, PowershellError> {
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

    /// Reads the current state of the tweak by executing the `read_script`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` with the current state if `read_script` is defined and succeeds.
    /// - `Ok(None)` if no `read_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    pub fn read_current_state(&self, id: TweakId) -> Result<Option<String>, PowershellError> {
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
                    PowershellError::ScriptExecutionError(format!(
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
                return Err(PowershellError::ScriptExecutionError(format!(
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
    }

    /// Executes the `apply_script` to apply the tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails.
    pub fn run_apply_script(&self, id: TweakId) -> Result<(), PowershellError> {
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
                        PowershellError::ScriptExecutionError(format!(
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
                    Err(PowershellError::ScriptExecutionError(format!(
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
    pub fn run_undo_script(&self, id: TweakId) -> Result<(), PowershellError> {
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
                    PowershellError::ScriptExecutionError(format!(
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
                return Err(PowershellError::ScriptExecutionError(format!(
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
    Tweak::new(
        TweakId::ProcessIdleTasks,
        "Process Idle Tasks".to_string(),
        "Forces the execution of scheduled background tasks that are normally run during system idle time. This helps free up system resources by completing these tasks immediately, improving overall system responsiveness and optimizing resource allocation. It can also reduce latency caused by deferred operations in critical system processes.".to_string(),
        TweakCategory::System,
         vec!["https://www.thewindowsclub.com/misconceptions-rundll32-exe-advapi32-dllprocessidletasks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: None,
            apply_script: Some("Rundll32.exe advapi32.dll,ProcessIdleTasks".to_string()),
            undo_script: None,
            target_state: None,
        }),
        false, // requires reboot
        TweakWidget::Button,
    )
}

pub fn enable_ultimate_performance_plan() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::UltimatePerformancePlan,
        "Enable Ultimate Performance Plan".to_string(),
        "Activates the Ultimate Performance power plan, which is tailored for demanding workloads by minimizing micro-latencies and boosting hardware performance. It disables power-saving features like core parking, hard disk sleep, and processor throttling, ensuring CPU cores run at maximum frequency. This plan also keeps I/O devices and PCIe links at full power, prioritizing performance over energy efficiency. It’s designed to reduce the delays introduced by energy-saving policies, improving responsiveness in tasks that require consistent, high-throughput system resources..".to_string(),
        TweakCategory::Power,
        vec!["https://www.elevenforum.com/t/restore-missing-power-plans-in-windows-11.6898/".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        false, // requires reboot
        TweakWidget::Switch,
    )
}

pub fn additional_kernel_worker_threads() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::AdditionalKernelWorkerThreads,
        "Additional Worker Threads".to_string(),
        "Increases the number of kernel worker threads by setting the AdditionalCriticalWorkerThreads and AdditionalDelayedWorkerThreads values to match the number of logical processors in the system. This tweak boosts performance in multi-threaded workloads by allowing the kernel to handle more concurrent operations, improving responsiveness and reducing bottlenecks in I/O-heavy or CPU-bound tasks. It ensures that both critical and delayed work items are processed more efficiently, particularly on systems with multiple cores.".to_string(),
        TweakCategory::Kernel,
        vec!["https://martin77s.wordpress.com/2010/04/05/performance-tuning-your-windows-server-part-3/".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_dynamic_tick() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableDynamicTick,
        "Disable Dynamic Tick".to_string(),
        "Disables the dynamic tick feature, which normally reduces timer interrupts during idle periods to conserve power. By disabling dynamic tick, the system maintains a constant rate of timer interrupts, improving performance in real-time applications by reducing latency and jitter. This tweak is useful in scenarios where consistent, low-latency processing is required, but it may increase power consumption as the CPU will not enter low-power states as frequently.".to_string(),
        TweakCategory::System,
        vec!["https://www.xbitlabs.com/how-to-get-better-latency-in-windows/".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some("bcdedit /enum | Select-String 'disabledynamictick'".to_string()),
            apply_script: Some("bcdedit /set disabledynamictick yes".to_string()),
            undo_script: Some("bcdedit /set disabledynamictick no".to_string()),
            target_state: Some("Yes".trim().to_string()),
        }),
        true,
        TweakWidget::Switch,
    )
}

pub fn aggressive_dpc_handling() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::AggressiveDpcHandling,
        "Aggressive DPC Handling".to_string(),
        "This tweak modifies kernel-level settings in the Windows Registry to aggressively optimize the handling of Deferred Procedure Calls (DPCs) by disabling timeouts, watchdogs, and minimizing queue depth, aiming to enhance system responsiveness and reduce latency. However, it also removes safeguards that monitor and control long-running DPCs, which could lead to system instability or crashes in certain scenarios, particularly during high-performance or overclocking operations.".to_string(),
        TweakCategory::Kernel,
        vec!["https://www.youtube.com/watch?v=4OrEytGFdK4".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn enhanced_kernel_performance() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::EnhancedKernelPerformance,
        "Enhanced Kernel Performance".to_string(),
        "Optimizes various kernel-level settings in the Windows Registry to improve system performance by increasing I/O queue sizes, buffer sizes, and stack sizes, while disabling certain security features. These changes aim to enhance multitasking and I/O operations but may affect system stability and security.".to_string(),
        TweakCategory::Kernel,
        vec!["https://www.youtube.com/watch?v=4OrEytGFdK4".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_ram_compression() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableRamCompression,
        "Disable RAM Compression".to_string(),
        "Disables the RAM compression feature in Windows to potentially improve system performance by reducing CPU overhead. This may lead to higher memory usage.".to_string(),
        TweakCategory::Memory,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        true,
        TweakWidget::Switch,
    )
}

pub fn disable_local_firewall() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableLocalFirewall,
        "Disable Local Firewall".to_string(),
        "Disables the local Windows Firewall for all profiles by setting the firewall state to `off`. **Warning:** This exposes the system to potential security threats and may cause issues with IPsec server connections.".to_string(),
        TweakCategory::Security,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        true,
        TweakWidget::Switch,
    )
}

pub fn disable_success_auditing() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableSuccessAuditing,
        "Disable Success Auditing".to_string(),
        "Disables auditing of successful events across all categories, reducing the volume of event logs and system overhead. Security events in the Windows Security log are not affected.".to_string(),
        TweakCategory::Security,
        vec!["https://sites.google.com/view/melodystweaks/securitytweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        true,
        TweakWidget::Switch,
    )
}

pub fn disable_pagefile() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisablePagefile,
        "Disable Pagefile".to_string(),
        "Disables the Windows page file, which is used as virtual memory when physical memory is full. This tweak can improve system performance by reducing disk I/O and preventing paging, but it may cause system instability or application crashes if the system runs out of memory.".to_string(),
        TweakCategory::Memory,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        true,
        TweakWidget::Switch,
    )
}

pub fn disable_speculative_execution_mitigations() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableSpeculativeExecutionMitigations,
        "Disable Speculative Execution Mitigations".to_string(),
        "Disables speculative execution mitigations by setting the `FeatureSettingsOverride` and `FeatureSettingsOverrideMask` registry values to `3`. This may improve performance but can also introduce security risks.".to_string(),
        TweakCategory::Security,
        vec!["https://www.tenforums.com/tutorials/5918-turn-off-data-execution-prevention-dep-windows.html".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        true,
        TweakWidget::Switch,
    )
}



pub fn disable_data_execution_prevention() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableDataExecutionPrevention,
        "Disable Data Execution Prevention".to_string(),
        "Disables Data Execution Prevention (DEP) by setting the `nx` boot configuration option to `AlwaysOff`. This may improve compatibility with older applications but can introduce security risks.".to_string(),
        TweakCategory::Security,
        vec!["https://www.tenforums.com/tutorials/5918-turn-off-data-execution-prevention-dep-windows.html".to_string()],
        TweakMethod::Powershell(PowershellTweak {
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
        }),
        true,
        TweakWidget::Switch,
    )
}

pub fn disable_process_idle_states()-> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableProcessIdleStates,
        "Disable Process Idle States".to_string(),
        "Disables processor idle states (C-states) to prevent the CPU from entering low-power states during idle periods. This tweak can improve system responsiveness but may increase power consumption and heat output.".to_string(),
        TweakCategory::Power,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $processorIdleStates = powercfg /query scheme_current sub_processor | Select-String 'Processor Idle Disable'
                
                if ($processorIdleStates -match 'Processor Idle Disable: 1') {
                    "Enabled"
                } else {
                    "Disabled"
                }
                "#
                .trim()
                .to_string(),
            ),
            apply_script: Some(
                "powercfg -setacvalueindex scheme_current sub_processor 5d76a2ca-e8c0-402f-a133-2158492d58ad 1".to_string(),
            ),
            undo_script: Some(
                "powercfg -setacvalueindex scheme_current sub_processor 5d76a2ca-e8c0-402f-a133-2158492d58ad 0".to_string(),
            ),
            target_state: Some("Enabled".to_string()),
        }),
        true,
        TweakWidget::Switch,
    )
}


pub fn kill_all_non_critical_services() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::KillAllNonCriticalServices,
        "Kill All Non-Critical Services".to_string(),
        "Stops all non-critical services to free up system resources and improve performance. This tweak may cause system instability or data loss.".to_string(),
        TweakCategory::System,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: None,
            apply_script: Some(
                r#"
                $services = @(
                    "AdobeARMservice",
                    "AdobeFlashPlayerUpdateSvc",
                    "AdobeUpdateService",
                    "AeLookupSvc",
                    "ALG",
                    "AppIDSvc",
                    "Appinfo",
                    "AppMgmt",
                    "AppReadiness",
                    "AppXSvc",
                    "AssignedAccessManagerSvc",
                    "AudioEndpointBuilder",
                    "Audiosrv",
                    "autotimesvc",
                    "AxInstSV",
                    "BDESVC",
                    "BFE",
                    "BITS",
                    "BrokerInfrastructure",
                    "Browser",
                    "BthAvctpSvc"
                )

                $failedServices = @()

                for ($i = 1; $i -le 5; $i++) {
                    Write-Output "Attempt $i to stop non-critical services..."
                    foreach ($service in $services) {
                        # Check if the service is already stopped
                        $serviceStatus = (Get-Service -Name $service -ErrorAction SilentlyContinue).Status
                        if ($serviceStatus -ne 'Stopped') {
                            try {
                                Stop-Service -Name $service -Force -ErrorAction Stop
                                Write-Output "Stopped service: $service"
                            } catch {
                                Write-Output "Failed to stop service: $service. Attempt $i/5."
                            }
                        } else {
                            Write-Output "Service already stopped: $service"
                        }
                    }
                    # Optional: Add a short delay between attempts
                    Start-Sleep -Seconds 2
                }

                # After all attempts, identify services that are still running
                foreach ($service in $services) {
                    $serviceStatus = (Get-Service -Name $service -ErrorAction SilentlyContinue).Status
                    if ($serviceStatus -ne 'Stopped') {
                        $failedServices += $service
                    }
                }

                if ($failedServices.Count -gt 0) {
                    Write-Output "The following services failed to be stopped after 5 attempts:"
                    foreach ($failedService in $failedServices) {
                        Write-Output "- $failedService"
                    }
                } else {
                    Write-Output "All specified non-critical services have been successfully stopped."
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                $services = @(
                    "AdobeARMservice",
                    "AdobeFlashPlayerUpdateSvc",
                    "AdobeUpdateService",
                    "AeLookupSvc",
                    "ALG",
                    "AppIDSvc",
                    "Appinfo",
                    "AppMgmt",
                    "AppReadiness",
                    "AppXSvc",
                    "AssignedAccessManagerSvc",
                    "AudioEndpointBuilder",
                    "Audiosrv",
                    "autotimesvc",
                    "AxInstSV",
                    "BDESVC",
                    "BFE",
                    "BITS",
                    "BrokerInfrastructure",
                    "Browser",
                    "BthAvctpSvc"
                )

                foreach ($service in $services) {
                    try {
                        Start-Service -Name $service -ErrorAction Stop
                        Write-Output "Started service: $service"
                    } catch {
                        Write-Output "Failed to start service: $service."
                    }
                }

                Write-Output "All non-critical services started."
                "#
                .trim()
                .to_string(),
            ),
            target_state: None,
        }),
        false,
        TweakWidget::Button,
    )
}
