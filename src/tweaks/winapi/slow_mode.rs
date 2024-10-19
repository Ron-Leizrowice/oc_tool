// src/tweaks/definitions/slow_mode.rs

use anyhow::{Context, Result};
use windows::core::GUID;

use crate::{
    power::{
        enumerate_power_subgroups_and_settings, get_active_power_scheme, read_ac_value_index,
        set_active_power_scheme, write_ac_value_index, PowerScheme, PowerSetting, PowerSubgroup,
    },
    tweaks::{TweakId, TweakMethod},
};

pub const POWER_SAVER_GUID: GUID = GUID::from_u128(0xa1841308_3541_4fab_bc81_f71556f20b4a);

// Desired values for Slow Mode settings
const SLOW_MODE_MAX_CORES: u32 = 2;
const SLOW_MODE_PERF_INC_THRESHOLD: u32 = 100; // Least aggressive
const SLOW_MODE_PERF_INC_TIME: u32 = 1; // Minimum
const SLOW_MODE_ENERGY_PERF_PREFERENCE: u32 = 0; // Maximum power saving
const SLOW_MODE_PROC_FREQ_MAX: u32 = 3000; // 3000 MHz

pub struct SlowMode {
    id: TweakId,
    settings: Vec<PowerSchemeSetting>,
    previous_power_scheme: PowerScheme,
}

#[derive(Debug)]
struct PowerSchemeSetting {
    subgroup_guid: GUID,
    power_setting_guid: GUID,
    value: u32,
}

/// Represents the names of power settings used in Slow Mode.
/// Modify these names based on your specific requirements and system settings.
const REQUIRED_POWER_SETTINGS: &[&str] = &[
    "Processor performance core parking min cores",
    "Processor performance increase threshold",
    "Processor performance increase time",
    "Energy/performance preference",
    "Processor maximum frequency",
];

impl SlowMode {
    /// Creates a new instance of `SlowMode`.
    pub fn new() -> Self {
        // Retrieve the current active power scheme
        let previous_power_scheme =
            get_active_power_scheme().expect("Failed to retrieve the current active power scheme.");

        // Enumerate all power subgroups and settings within the active power scheme
        let power_subgroups = enumerate_power_subgroups_and_settings(&previous_power_scheme.guid)
            .expect("Failed to enumerate power subgroups and settings.");

        // Define the settings to apply in Slow Mode by matching setting names
        let mut settings = Vec::new();

        for required_name in REQUIRED_POWER_SETTINGS {
            // Find the setting by name
            if let Some(setting) = find_setting_by_name(&power_subgroups, required_name) {
                // Assign the desired value based on the setting's purpose
                let value = match *required_name {
                    "Processor performance core parking min cores" => SLOW_MODE_MAX_CORES,
                    "Processor performance increase threshold" => SLOW_MODE_PERF_INC_THRESHOLD,
                    "Processor performance increase time" => SLOW_MODE_PERF_INC_TIME,
                    "Energy/performance preference" => SLOW_MODE_ENERGY_PERF_PREFERENCE,
                    "Processor maximum frequency" => SLOW_MODE_PROC_FREQ_MAX,
                    _ => continue, // Skip if the setting is not recognized
                };

                settings.push(PowerSchemeSetting {
                    subgroup_guid: setting.guid,
                    power_setting_guid: setting.guid,
                    value,
                });
            } else {
                tracing::warn!(
                    "{:?}-> Required power setting '{}' not found.",
                    TweakId::SlowMode,
                    required_name
                );
            }
        }

        Self {
            id: TweakId::SlowMode,
            settings,
            previous_power_scheme,
        }
    }
}

/// Finds a power setting by its display name.
fn find_setting_by_name<'a>(
    subgroups: &'a [PowerSubgroup],
    name: &str,
) -> Option<&'a PowerSetting> {
    for subgroup in subgroups {
        for setting in &subgroup.settings {
            if setting.name.eq_ignore_ascii_case(name) {
                return Some(setting);
            }
        }
    }
    None
}

impl TweakMethod for SlowMode {
    /// Checks if Slow Mode is currently enabled.
    ///
    /// This is determined by verifying if the active power scheme is Power Saver
    /// and if all the specified settings match the desired Slow Mode values.
    fn initial_state(&self) -> Result<bool, anyhow::Error> {
        tracing::debug!("{:?}-> Checking initial state", self.id);

        // Retrieve the current active power scheme
        let active_scheme = get_active_power_scheme()
            .context(format!("{:?}-> Failed to get active power scheme", self.id))?;

        // Check if the active scheme is Power Saver
        if active_scheme.guid != POWER_SAVER_GUID {
            tracing::debug!("{:?}-> Active scheme is not Power Saver.", self.id);
            return Ok(false);
        }

        // For each setting, verify if it matches the Slow Mode value
        for setting in &self.settings {
            let current_value = read_ac_value_index(
                &active_scheme.guid,
                &setting.subgroup_guid,
                &setting.power_setting_guid,
            )
            .with_context(|| {
                format!(
                    "{:?}-> Failed to read setting value for subgroup {:?} and power setting {:?}",
                    self.id, setting.subgroup_guid, setting.power_setting_guid
                )
            })?;

            if current_value != setting.value {
                tracing::debug!(
                    "{:?}-> Setting mismatch: expected {}, found {}.",
                    self.id,
                    setting.value,
                    current_value
                );
                return Ok(false);
            }
        }

        tracing::debug!("{:?}-> Initial state is Enabled", self.id);
        Ok(true)
    }

    /// Applies the Slow Mode tweak.
    ///
    /// This involves switching to the Power Saver scheme and applying the specified settings.
    fn apply(&self) -> Result<(), anyhow::Error> {
        tracing::debug!("{:?}-> Applying Slow Mode", self.id);

        // Apply each setting
        for setting in &self.settings {
            write_ac_value_index(
                &POWER_SAVER_GUID,
                &setting.subgroup_guid,
                &setting.power_setting_guid,
                setting.value,
            )
            .with_context(|| {
                format!(
                    "{:?}-> Failed to apply setting for subgroup {:?} and power setting {:?}",
                    self.id, setting.subgroup_guid, setting.power_setting_guid
                )
            })?;
        }

        // Activate the Power Saver scheme to apply changes
        set_active_power_scheme(&POWER_SAVER_GUID)
            .with_context(|| format!("{:?}-> Failed to activate Power Saver scheme", self.id))?;

        tracing::debug!("{:?}-> Slow Mode applied", self.id);
        Ok(())
    }

    /// Reverts the Slow Mode tweak.
    ///
    /// This restores the previously active power scheme.
    fn revert(&self) -> Result<(), anyhow::Error> {
        tracing::debug!("{:?}-> Reverting Slow Mode", self.id);

        // Restore the previous power scheme
        set_active_power_scheme(&self.previous_power_scheme.guid)
            .with_context(|| format!("{:?}-> Failed to activate previous power scheme", self.id))?;

        tracing::debug!("{:?}-> Slow Mode reverted", self.id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slow_mode_apply_and_revert() {
        // Initialize SlowMode
        let slow_mode = SlowMode::new();

        // Ensure initial state is not enabled
        let initial = slow_mode
            .initial_state()
            .expect("Failed to get initial state");
        if initial {
            // If already in Slow Mode, revert first
            slow_mode.revert().expect("Failed to revert SlowMode");
        }

        // Apply Slow Mode
        slow_mode.apply().expect("Failed to apply SlowMode");

        // Check if Slow Mode is enabled
        let applied = slow_mode
            .initial_state()
            .expect("Failed to verify SlowMode application");
        assert!(applied, "Slow Mode should be enabled after applying");

        // Revert Slow Mode
        slow_mode.revert().expect("Failed to revert SlowMode");

        // Check if Slow Mode is disabled
        let reverted = slow_mode
            .initial_state()
            .expect("Failed to verify Slow Mode reversion");
        assert!(!reverted, "Slow Mode should be disabled after reverting");
    }
}
