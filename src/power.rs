// src/power.rs

use std::process::Command;

use anyhow::{anyhow, Context, Result as AnyResult};

use crate::MyApp;

const SLOW_MODE_CMD: &str = r#"
powercfg /SETACVALUEINDEX SCHEME_MAX SUB_PROCESSOR CPMAXCORES 2
powercfg /SETDCVALUEINDEX SCHEME_MAX SUB_PROCESSOR CPMAXCORES1 2
powercfg /SETACVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFINCTHRESHOLD 100
powercfg /SETDCVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFINCTHRESHOLD1 100
powercfg /SETACVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFINCTIME 1
powercfg /SETDCVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFINCTIME1 1
powercfg /SETACVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFEPP 100
powercfg /SETDCVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFEPP1 100
powercfg /SETACVALUEINDEX SCHEME_MAX SUB_PROCESSOR PROCTHROTTLEMAX 1
powercfg /SETDCVALUEINDEX SCHEME_MAX SUB_PROCESSOR PROCTHROTTLEMAX1 1
powercfg /SETACVALUEINDEX SCHEME_MAX SUB_PROCESSOR PROCFREQMAX 1000
powercfg /SETDCVALUEINDEX SCHEME_MAX SUB_PROCESSOR PROCFREQMAX1 1000
powercfg /SETACVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFINCTHRESHOLD 100
powercfg /SETDCVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFINCTHRESHOLD1 100
powercfg /SETACVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFEPP 100
powercfg /SETDCVALUEINDEX SCHEME_MAX SUB_PROCESSOR PERFEPP1 100
powercfg -setactive SCHEME_MAX
"#;

pub const SLOW_MODE_DESCRIPTION: &str = "Places the system in a low-power state by:
1. Switching to the Power Saver scheme
2. Limiting max cores to 2
3. Limiting CPU frequency
4. Delaying CPU performance state transitions
";

/// Represents the various power states available.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PowerState {
    Balanced,
    HighPerformance,
    PowerSaver,
    UltimatePerformance,
    Unknown(String),
}

impl PowerState {
    fn as_str(&self) -> &str {
        match self {
            PowerState::Balanced => "SCHEME_BALANCED",
            PowerState::HighPerformance => "SCHEME_MIN",
            PowerState::PowerSaver => "SCHEME_MAX",
            PowerState::UltimatePerformance => "SUB_DISK",
            PowerState::Unknown(name) => name,
        }
    }
}

/// Trait for managing slow mode functionality.
pub trait SlowMode {
    /// Enables slow mode by switching to Power Saver and applying additional constraints.
    fn enable_slow_mode(&mut self) -> AnyResult<()>;

    /// Disables slow mode by reverting to the previously active power state.
    fn disable_slow_mode(&mut self) -> AnyResult<()>;
}

impl SlowMode for MyApp {
    fn enable_slow_mode(&mut self) -> AnyResult<()> {
        let output = Command::new("powercfg")
            .args(["/setactive", PowerState::PowerSaver.as_str()])
            .output()
            .context("Failed to execute 'powercfg /setactive' command")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Command 'powercfg /setactive' failed with status: {}",
                output.status
            ));
        }

        Command::new("powershell")
            .args(["-Command", SLOW_MODE_CMD])
            .output()
            .context("Failed to execute PowerShell command")?;

        println!("Slow mode enabled successfully.");
        Ok(())
    }

    fn disable_slow_mode(&mut self) -> AnyResult<()> {
        Command::new("powercfg")
            .args(["/setactive", self.power_state.as_str()])
            .output()
            .context("Failed to execute 'powercfg /setactive' command")?;

        tracing::info!("Slow mode disabled successfully.");
        Ok(())
    }
}

/// Reads the current active power state.
///
/// # Returns
///
/// The current `PowerState`.
pub fn read_power_state() -> AnyResult<PowerState> {
    // Execute the PowerShell command to get the active power scheme
    let output = Command::new("powercfg")
        .args(["/GetActiveScheme"])
        .output()
        .context("Failed to execute 'powercfg /getactivescheme' command")?;

    if !output.status.success() {
        return Err(anyhow!(
            "Command 'powercfg /getactivescheme' failed with status: {}",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // check if it contains "Balanced"
    if stdout.contains("Balanced") {
        Ok(PowerState::Balanced)
    } else if stdout.contains("High performance") {
        return Ok(PowerState::HighPerformance);
    } else if stdout.contains("Power saver") {
        return Ok(PowerState::PowerSaver);
    } else if stdout.contains("Ultimate performance") {
        return Ok(PowerState::UltimatePerformance);
    } else {
        return Ok(PowerState::Unknown("Unknown".to_string()));
    }
}
