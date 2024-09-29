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
        false,
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
        false,
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
        false,
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

pub fn thread_dpc_disable() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::ThreadDpcDisable,
        "Thread DPC Disable".to_string(),
        "Disables or modifies the handling of Deferred Procedure Calls (DPCs) related to threads by setting the 'ThreadDpcEnable' registry value to 0. This aims to reduce DPC overhead and potentially enhance system responsiveness. However, it may lead to system instability or compatibility issues with certain hardware or drivers.".to_string(),
        TweakCategory::Kernel,
        vec!["https://www.youtube.com/watch?v=4OrEytGFdK4".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"
                try {
                    $threadDpcEnable = (Get-ItemProperty -Path $path -Name "ThreadDpcEnable" -ErrorAction SilentlyContinue).ThreadDpcEnable

                    if ($threadDpcEnable -eq 0) {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                } catch {
                    Write-Error "Failed to read the ThreadDpcEnable registry value."
                }
                "#
                .trim()
                .to_string(),
            ),
            apply_script: Some(
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"
                try {
                    Set-ItemProperty -Path $path -Name "ThreadDpcEnable" -Value 0 -Type DWord -Force
                    Write-Output "Thread DPC Disable Tweak Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Thread DPC Disable Tweak: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"
                try {
                    Remove-ItemProperty -Path $path -Name "ThreadDpcEnable" -ErrorAction SilentlyContinue
                    Write-Output "Thread DPC Disable Tweak Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Thread DPC Disable Tweak: $_"
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

pub fn disable_5_level_paging() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::Disable5LevelPaging,
        "Disable 5-Level Paging and Increase User Virtual Memory".to_string(),
        "Disables 57-bit 5-level paging (Linear Address 57) for 10th Gen Intel CPUs and increases the user-mode virtual address space to 256 TB per disk. This tweak is effective only on compatible Intel CPUs and aims to optimize memory usage for high-performance scenarios.".to_string(),
        TweakCategory::Memory,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $linearAddress57 = bcdedit /enum | Select-String "linearaddress57" | ForEach-Object { $_.Line }
                $increaseUserVA = bcdedit /enum | Select-String "increaseuserva" | ForEach-Object { $_.Line }
                
                if ($linearAddress57 -match "OptOut" -and $increaseUserVA -match "268435328") {
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
                    bcdedit /set linearaddress57 OptOut
                    bcdedit /set increaseuserva 268435328
                    Write-Output "Disable 5-Level Paging and Increase User Virtual Memory Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable 5-Level Paging and Increase User Virtual Memory Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    bcdedit /deletevalue linearaddress57
                    bcdedit /deletevalue increaseuserva
                    Write-Output "Disable 5-Level Paging and Increase User Virtual Memory Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable 5-Level Paging and Increase User Virtual Memory Tweaks: $_"
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

pub fn optimize_memory_allocation() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::OptimizeMemoryAllocation,
        "Optimize Memory Allocation".to_string(),
        "Avoids the use of uncontiguous low-memory portions by setting memory allocation policies. This tweak boosts memory performance and improves microstuttering in approximately 80% of cases. However, it may cause system freezes if memory modules are unstable.".to_string(),
        TweakCategory::Memory,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $firstMBPolicy = bcdedit /enum | Select-String "firstmegabytepolicy" | ForEach-Object { $_.Line }
                $avoidLowMemory = bcdedit /enum | Select-String "avoidlowmemory" | ForEach-Object { $_.Line }
                $noLowMem = bcdedit /enum | Select-String "nolowmem" | ForEach-Object { $_.Line }
                
                if ($firstMBPolicy -match "UseAll" -and $avoidLowMemory -match "0x8000000" -and $noLowMem -match "Yes") {
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
                    bcdedit /set firstmegabytepolicy UseAll
                    bcdedit /set avoidlowmemory 0x8000000
                    bcdedit /set nolowmem Yes
                    Write-Output "Optimize Memory Allocation Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Optimize Memory Allocation Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    bcdedit /deletevalue firstmegabytepolicy
                    bcdedit /deletevalue avoidlowmemory
                    bcdedit /deletevalue nolowmem
                    Write-Output "Optimize Memory Allocation Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Optimize Memory Allocation Tweaks: $_"
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

pub fn disable_kernel_memory_mitigations() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableKernelMemoryMitigations,
        "Disable Kernel Memory Mitigations".to_string(),
        "Disables specific kernel memory mitigations to enhance performance. This tweak may cause boot crashes or loops if Intel SGX is enforced and not set to 'Application Controlled' or 'Off' in the firmware. It's recommended only for systems where SGX is not utilized.".to_string(),
        TweakCategory::Security,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $allowedInMemSettings = bcdedit /enum | Select-String "allowedinmemorysettings" | ForEach-Object { $_.Line }
                $isolatedContext = bcdedit /enum | Select-String "isolatedcontext" | ForEach-Object { $_.Line }
                
                if ($allowedInMemSettings -match "0x0" -and $isolatedContext -match "No") {
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
                    bcdedit /set allowedinmemorysettings 0x0
                    bcdedit /set isolatedcontext No
                    Write-Output "Disable Kernel Memory Mitigations Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable Kernel Memory Mitigations Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    bcdedit /deletevalue allowedinmemorysettings
                    bcdedit /deletevalue isolatedcontext
                    Write-Output "Disable Kernel Memory Mitigations Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable Kernel Memory Mitigations Tweaks: $_"
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

pub fn disable_dma_protection() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableDMAProtection,
        "Disable DMA Protection and Core Isolation".to_string(),
        "Disables DMA memory protection and core isolation (virtualization-based protection) to enhance system performance. This tweak may reduce security by allowing direct memory access and weakening core isolation protections.".to_string(),
        TweakCategory::Security,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $vsmLaunchType = bcdedit /enum | Select-String "vsmlaunchtype" | ForEach-Object { $_.Line }
                $vmSetting = bcdedit /enum | Select-String "vm" | ForEach-Object { $_.Line }
                
                $fvePath = "HKLM:\SOFTWARE\Policies\Microsoft\FVE"
                $deviceGuardPath = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard"
                
                $disableExternalDMA = (Get-ItemProperty -Path $fvePath -Name "DisableExternalDMAUnderLock" -ErrorAction SilentlyContinue).DisableExternalDMAUnderLock
                $enableVBS = (Get-ItemProperty -Path $deviceGuardPath -Name "EnableVirtualizationBasedSecurity" -ErrorAction SilentlyContinue).EnableVirtualizationBasedSecurity
                $hvCIMATRequired = (Get-ItemProperty -Path $deviceGuardPath -Name "HVCIMATRequired" -ErrorAction SilentlyContinue).HVCIMATRequired
                
                if ($vsmLaunchType -match "Off" -and $vmSetting -match "No" -and $disableExternalDMA -eq 0 -and $enableVBS -eq 0 -and $hvCIMATRequired -eq 0) {
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
                    bcdedit /set vsmlaunchtype Off
                    bcdedit /set vm No

                    New-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\FVE" -Name "DisableExternalDMAUnderLock" -Value 0 -PropertyType DWord -Force | Out-Null
                    New-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" -Name "EnableVirtualizationBasedSecurity" -Value 0 -PropertyType DWord -Force | Out-Null
                    New-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" -Name "HVCIMATRequired" -Value 0 -PropertyType DWord -Force | Out-Null

                    Write-Output "Disable DMA Protection and Core Isolation Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable DMA Protection and Core Isolation Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    bcdedit /deletevalue vsmlaunchtype
                    bcdedit /deletevalue vm

                    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\FVE" -Name "DisableExternalDMAUnderLock" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" -Name "EnableVirtualizationBasedSecurity" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" -Name "HVCIMATRequired" -ErrorAction SilentlyContinue

                    Write-Output "Disable DMA Protection and Core Isolation Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable DMA Protection and Core Isolation Tweaks: $_"
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

pub fn disable_process_kernel_mitigations() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableProcessKernelMitigations,
        "Disable Process and Kernel Mitigations".to_string(),
        "Disables several process and kernel mitigations to improve performance. This includes disabling exception chain validation, SEHOP, CFG, and removing Image File Execution Options. **Warning:** These changes can significantly weaken system security and may cause instability or crashes.".to_string(),
        TweakCategory::Security,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $disableExceptionChainValidation = (Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name "DisableExceptionChainValidation" -ErrorAction SilentlyContinue).DisableExceptionChainValidation
                $kernelSEHOPEnabled = (Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name "KernelSEHOPEnabled" -ErrorAction SilentlyContinue).KernelSEHOPEnabled
                $enableCfg = (Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name "EnableCfg" -ErrorAction SilentlyContinue).EnableCfg
                
                $imageFileExecOptions = Get-ChildItem -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options" -ErrorAction SilentlyContinue
                
                if ($disableExceptionChainValidation -eq 1 -and $kernelSEHOPEnabled -eq 0 -and $enableCfg -eq 0 -and $imageFileExecOptions -eq $null) {
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
                    # Disable Exception Chain Validation and SEHOP
                    Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name "DisableExceptionChainValidation" -Value 1 -Type DWord -Force
                    Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name "KernelSEHOPEnabled" -Value 0 -Type DWord -Force
                    
                    # Disable Control Flow Guard
                    Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name "EnableCfg" -Value 0 -Type DWord -Force
                    
                    # Remove Image File Execution Options
                    Remove-Item -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options" -Recurse -ErrorAction SilentlyContinue
                    
                    Write-Output "Disable Process and Kernel Mitigations Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable Process and Kernel Mitigations Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    # Re-enable Exception Chain Validation and SEHOP
                    Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name "DisableExceptionChainValidation" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name "KernelSEHOPEnabled" -ErrorAction SilentlyContinue
                    
                    # Re-enable Control Flow Guard
                    Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" -Name "EnableCfg" -ErrorAction SilentlyContinue
                    
                    Write-Output "Disable Process and Kernel Mitigations Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable Process and Kernel Mitigations Tweaks: $_"
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

pub fn realtime_priority_csrss() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::RealtimePriorityCsrss,
        "Set Realtime Priority for csrss.exe".to_string(),
        "Configures the `csrss.exe` process to use realtime CPU and I/O priorities, aiming to improve system performance and responsiveness. **Caution:** Improper configuration can lead to system instability.".to_string(),
        TweakCategory::System,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $csrssPerfOptions = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions" -ErrorAction SilentlyContinue
                if ($csrssPerfOptions.CpuPriorityClass -eq 4 -and $csrssPerfOptions.IoPriority -eq 3) {
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
                    New-Item -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions" -Force | Out-Null
                    Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions" -Name "CpuPriorityClass" -Value 4 -Type DWord
                    Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions" -Name "IoPriority" -Value 3 -Type DWord
                    Write-Output "Set Realtime Priority for csrss.exe Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Set Realtime Priority for csrss.exe Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions" -Name "CpuPriorityClass" -ErrorAction SilentlyContinue
                    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions" -Name "IoPriority" -ErrorAction SilentlyContinue
                    Write-Output "Set Realtime Priority for csrss.exe Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Set Realtime Priority for csrss.exe Tweaks: $_"
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
        false,
        TweakWidget::Switch,
    )
}

pub fn disable_ntfs_refs_mitigations() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableNTFSREFSMitigations,
        "Disable Additional NTFS/ReFS Mitigations".to_string(),
        "Disables additional mitigations for NTFS and ReFS file systems by setting the `ProtectionMode` registry value to `0`. This can improve file system performance but may reduce security and data integrity features.".to_string(),
        TweakCategory::Security,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"
                $protectionMode = (Get-ItemProperty -Path $path -Name "ProtectionMode" -ErrorAction SilentlyContinue).ProtectionMode
                
                if ($protectionMode -eq 0) {
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
                    Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name "ProtectionMode" -Value 0 -Type DWord -Force
                    Write-Output "Disable Additional NTFS/ReFS Mitigations Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable Additional NTFS/ReFS Mitigations Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\kernel" -Name "ProtectionMode" -ErrorAction SilentlyContinue
                    Write-Output "Disable Additional NTFS/ReFS Mitigations Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable Additional NTFS/ReFS Mitigations Tweaks: $_"
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

pub fn enable_x2apic_memory_mapping() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::EnableX2ApicMemoryMapping,
        "Enable X2Apic and Memory Mapping for PCI-E Devices".to_string(),
        "Enables X2APIC and memory mapping for PCI-E devices by setting various BCDEdit options. For optimal results, MSI mode should be enabled for all devices using the MSI utility or manually. This tweak enhances CPU and device communication but requires careful configuration of device settings.".to_string(),
        TweakCategory::System,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $x2apicPolicy = bcdedit /enum | Select-String "x2apicpolicy" | ForEach-Object { $_.Line }
                $configAccessPolicy = bcdedit /enum | Select-String "configaccesspolicy" | ForEach-Object { $_.Line }
                $msiSetting = bcdedit /enum | Select-String "MSI" | ForEach-Object { $_.Line }
                $usePhysicalDestination = bcdedit /enum | Select-String "usephysicaldestination" | ForEach-Object { $_.Line }
                $useFirmwarePCISettings = bcdedit /enum | Select-String "usefirmwarepcisettings" | ForEach-Object { $_.Line }
                
                if ($x2apicPolicy -match "Enable" -and $configAccessPolicy -match "Default" -and $msiSetting -match "Default" -and $usePhysicalDestination -match "No" -and $useFirmwarePCISettings -match "No") {
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
                    bcdedit /set x2apicpolicy Enable
                    bcdedit /set configaccesspolicy Default
                    bcdedit /set MSI Default
                    bcdedit /set usephysicaldestination No
                    bcdedit /set usefirmwarepcisettings No
                    Write-Output "Enable X2Apic and Memory Mapping for PCI-E Devices Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Enable X2Apic and Memory Mapping Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    bcdedit /deletevalue x2apicpolicy
                    bcdedit /deletevalue configaccesspolicy
                    bcdedit /deletevalue MSI
                    bcdedit /deletevalue usephysicaldestination
                    bcdedit /deletevalue usefirmwarepcisettings
                    Write-Output "Enable X2Apic and Memory Mapping for PCI-E Devices Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Enable X2Apic and Memory Mapping Tweaks: $_"
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

pub fn force_contiguous_memory_dx_kernel() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::ForceContiguousMemoryDxKernel,
        "Force Contiguous Memory Allocation in DirectX Graphics Kernel".to_string(),
        "Forces the DirectX Graphics Kernel to allocate memory contiguously by setting the `DpiMapIommuContiguous` registry value to `1`. This tweak aims to reduce microstuttering and improve graphics performance.".to_string(),
        TweakCategory::Graphics,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $path = "HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers"
                $dpiMapIommuContiguous = (Get-ItemProperty -Path $path -Name "DpiMapIommuContiguous" -ErrorAction SilentlyContinue).DpiMapIommuContiguous
                
                if ($dpiMapIommuContiguous -eq 1) {
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
                    Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers" -Name "DpiMapIommuContiguous" -Value 1 -Type DWord -Force
                    Write-Output "Force Contiguous Memory Allocation in DirectX Graphics Kernel Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Force Contiguous Memory Allocation in DirectX Graphics Kernel Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    Remove-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers" -Name "DpiMapIommuContiguous" -ErrorAction SilentlyContinue
                    Write-Output "Force Contiguous Memory Allocation in DirectX Graphics Kernel Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Force Contiguous Memory Allocation in DirectX Graphics Kernel Tweaks: $_"
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

pub fn force_contiguous_memory_nvidia() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::ForceContiguousMemoryNvidia,
        "Force Contiguous Memory Allocation in NVIDIA Driver".to_string(),
        "Forces the NVIDIA driver to allocate memory contiguously by setting the `PreferSystemMemoryContiguous` registry value to `1`. This can improve graphics performance but requires specifying the correct device path (e.g., `0000`).".to_string(),
        TweakCategory::Graphics,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $gpuKey = Get-ChildItem -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}" | Select-Object -First 1
                $preferSystemMemContiguous = (Get-ItemProperty -Path $gpuKey.PSPath -Name "PreferSystemMemoryContiguous" -ErrorAction SilentlyContinue).PreferSystemMemoryContiguous
                
                if ($preferSystemMemContiguous -eq 1) {
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
                    $gpuKey = Get-ChildItem -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}" | Select-Object -First 1
                    Set-ItemProperty -Path $gpuKey.PSPath -Name "PreferSystemMemoryContiguous" -Value 1 -Type DWord -Force
                    Write-Output "Force Contiguous Memory Allocation in NVIDIA Driver Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Force Contiguous Memory Allocation in NVIDIA Driver Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    $gpuKey = Get-ChildItem -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}" | Select-Object -First 1
                    Remove-ItemProperty -Path $gpuKey.PSPath -Name "PreferSystemMemoryContiguous" -ErrorAction SilentlyContinue
                    Write-Output "Force Contiguous Memory Allocation in NVIDIA Driver Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Force Contiguous Memory Allocation in NVIDIA Driver Tweaks: $_"
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

pub fn disable_application_telemetry() -> Arc<Mutex<Tweak>> {
    Tweak::new(
        TweakId::DisableApplicationTelemetry,
        "Disable Application Telemetry".to_string(),
        "Disables Windows Application Telemetry by setting the `AITEnable` registry value to `0`. This reduces the collection of application telemetry data but may limit certain features or diagnostics.".to_string(),
        TweakCategory::Telemetry,
        vec!["https://sites.google.com/view/melodystweaks/basictweaks".to_string()],
        TweakMethod::Powershell(PowershellTweak {
            read_script: Some(
                r#"
                $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\AppCompat"
                $aitEnable = (Get-ItemProperty -Path $path -Name "AITEnable" -ErrorAction SilentlyContinue).AITEnable
                
                if ($aitEnable -eq 0) {
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
                    New-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\AppCompat" -Name "AITEnable" -Value 0 -PropertyType DWord -Force | Out-Null
                    Write-Output "Disable Application Telemetry Applied Successfully."
                } catch {
                    Write-Error "Failed to apply Disable Application Telemetry Tweaks: $_"
                }
                "#
                .trim()
                .to_string(),
            ),
            undo_script: Some(
                r#"
                try {
                    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\AppCompat" -Name "AITEnable" -ErrorAction SilentlyContinue
                    Write-Output "Disable Application Telemetry Reverted Successfully."
                } catch {
                    Write-Error "Failed to revert Disable Application Telemetry Tweaks: $_"
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
        false,
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
        false,
        TweakWidget::Switch,
    )
}
