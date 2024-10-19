use std::ptr::null_mut;

use anyhow::Error;
use tracing::{error, info};
use windows::{
    core::GUID,
    Win32::System::{
        Power::{
            PowerGetActiveScheme, PowerReadACValueIndex, PowerSetActiveScheme,
            PowerWriteACValueIndex,
        },
        Registry::HKEY,
        SystemServices::{GUID_PROCESSOR_IDLE_DISABLE, GUID_PROCESSOR_SETTINGS_SUBGROUP},
    },
};

use crate::tweaks::{TweakId, TweakMethod};

pub struct DisableProcessIdleStates {
    id: TweakId,
}

impl DisableProcessIdleStates {
    pub fn new() -> Self {
        Self {
            id: TweakId::DisableProcessIdleStates,
        }
    }

    /// Retrieves the current active power scheme GUID.
    fn get_current_scheme() -> Result<*mut GUID, Error> {
        let mut active_policy_guid = null_mut();
        let result = unsafe { PowerGetActiveScheme(None, &mut active_policy_guid) };

        if result.is_err() {
            error!("Failed to get active power scheme: {:?}", result);
            return Err(Error::msg(format!(
                "PowerGetActiveScheme failed with error: {:?}",
                result
            )));
        }

        info!(
            "Retrieved current power scheme GUID: {:?}",
            active_policy_guid
        );
        Ok(active_policy_guid)
    }

    /// Reads the current value of the Processor Idle Disable setting.
    fn read_idle_disable_value() -> Result<u32, Error> {
        let active_policy_guid = Self::get_current_scheme()?;
        let mut value = 0u32;
        let result = unsafe {
            PowerReadACValueIndex(
                HKEY::default(),
                Some(active_policy_guid),
                Some(&GUID_PROCESSOR_SETTINGS_SUBGROUP),
                Some(&GUID_PROCESSOR_IDLE_DISABLE),
                &mut value,
            )
        };

        if result.is_err() {
            error!("Failed to read Processor Idle Disable value: {:?}", result);
            return Err(Error::msg(format!(
                "PowerReadACValueIndex failed with error: {:?}",
                result
            )));
        }

        info!(
            "Current Processor Idle Disable value: {}",
            if value == 1 { "Enabled" } else { "Disabled" }
        );
        Ok(value)
    }

    /// Writes a new value to the Processor Idle Disable setting.
    fn write_idle_disable_value(value: u32) -> Result<(), Error> {
        let active_policy_guid = Self::get_current_scheme()?;
        let write_result = unsafe {
            PowerWriteACValueIndex(
                None,
                active_policy_guid,
                Some(&GUID_PROCESSOR_SETTINGS_SUBGROUP),
                Some(&GUID_PROCESSOR_IDLE_DISABLE),
                value,
            )
        };

        if write_result.is_err() {
            error!(
                "Failed to write Processor Idle Disable value to {}: {:?}",
                value, write_result
            );
            return Err(Error::msg(format!(
                "PowerWriteACValueIndex failed with error: {:?}",
                write_result
            )));
        }

        let set_result = unsafe { PowerSetActiveScheme(None, Some(active_policy_guid)) };

        if set_result.is_err() {
            error!(
                "Failed to set active power scheme after writing value: {:?}",
                set_result
            );
            return Err(Error::msg(format!(
                "PowerSetActiveScheme failed with error: {:?}",
                set_result
            )));
        }

        info!(
            "Processor Idle Disable value successfully set to {}",
            if value == 1 { "Enabled" } else { "Disabled" }
        );
        Ok(())
    }
}

impl TweakMethod for DisableProcessIdleStates {
    /// Retrieves the initial state of the Processor Idle Disable setting.
    fn initial_state(&self) -> Result<bool, Error> {
        info!(
            "{:?} -> Checking initial state of Processor Idle Disable.",
            self.id
        );
        match Self::read_idle_disable_value() {
            Ok(value) => {
                let is_disabled = value == 1;
                info!(
                    "{:?} -> Initial state: Processor Idle Disable is {}.",
                    self.id,
                    if is_disabled { "Enabled" } else { "Disabled" }
                );
                Ok(is_disabled)
            }
            Err(e) => {
                error!(
                    "{:?} -> Failed to read initial Processor Idle Disable value: {:?}",
                    self.id, e
                );
                Err(e)
            }
        }
    }

    /// Applies the tweak by disabling Processor Idle States.
    fn apply(&self) -> Result<(), Error> {
        info!(
            "{:?} -> Applying tweak: Disable Processor Idle States.",
            self.id
        );
        Self::write_idle_disable_value(1).map_err(|e| {
            error!(
                "{:?} -> Failed to disable Processor Idle States: {:?}",
                self.id, e
            );
            e
        })?;
        info!(
            "{:?} -> Successfully disabled Processor Idle States.",
            self.id
        );
        Ok(())
    }

    /// Reverts the tweak by enabling Processor Idle States.
    fn revert(&self) -> Result<(), Error> {
        info!(
            "{:?} -> Reverting tweak: Enable Processor Idle States.",
            self.id
        );
        Self::write_idle_disable_value(0).map_err(|e| {
            error!(
                "{:?} -> Failed to enable Processor Idle States: {:?}",
                self.id, e
            );
            e
        })?;
        info!(
            "{:?} -> Successfully enabled Processor Idle States.",
            self.id
        );
        Ok(())
    }
}
