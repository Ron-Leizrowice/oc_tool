// src/tweaks/rust.rs

use std::process::Command;

use anyhow::Error;
use DisplaySettings::{get_display_settings, set_display_settings, DisplaySettingsType};

use super::{Tweak, TweakCategory, TweakId, TweakMethod};
use crate::widgets::TweakWidget;

pub struct LowResMode {
    pub default: DisplaySettingsType,
    pub target_state: DisplaySettingsType,
}

impl Default for LowResMode {
    fn default() -> Self {
        let target_state: DisplaySettingsType = {
            let options = get_display_settings();
            if let Some(valid_state) = options.iter().find(|x| x.refresh_rate == 30) {
                valid_state.clone()
            } else if let Some(valid_state) =
                options.iter().find(|x| x.width == 800 && x.height == 600)
            {
                valid_state.clone()
            } else {
                options.first().unwrap().clone()
            }
        };
        Self {
            default: get_display_settings().last().unwrap().clone(),
            target_state,
        }
    }
}

impl TweakMethod for LowResMode {
    fn initial_state(&self, id: TweakId) -> Result<bool, anyhow::Error> {
        let binding = get_display_settings();
        let current = binding.last().unwrap();
        tracing::info!("{:?} -> Current display settings: {:?}", id, current);
        Ok(current == &self.target_state)
    }

    fn apply(&self, id: TweakId) -> Result<(), anyhow::Error> {
        let result = set_display_settings(self.target_state.clone());
        match result {
            0 => Ok(()),
            _ => Err(anyhow::anyhow!(
                "{:?} -> Failed to apply display settings. Error code: {}",
                id,
                result
            )),
        }
    }

    fn revert(&self, id: TweakId) -> Result<(), anyhow::Error> {
        let result = set_display_settings(self.default.clone());
        match result {
            0 => Ok(()),
            _ => Err(anyhow::anyhow!(
                "{:?} -> Failed to revert display settings. Error code: {}",
                id,
                result
            )),
        }
    }
}

/// Function to create the `Low Resolution Mode` Rust tweak.
pub fn low_res_mode() -> Tweak {
    Tweak::rust_tweak(
        "Low Resolution Mode".to_string(),
        "Sets the display to a lower resolution to conserve resources or improve performance."
            .to_string(),
        TweakCategory::Graphics,
        LowResMode::default(),
        TweakWidget::Toggle,
        false,
    )
}

pub fn process_idle_tasks() -> Tweak {
    Tweak::rust_tweak(
        "Process Idle Tasks".to_string(),
        "Forces the execution of scheduled background tasks that are normally run during system idle time. This helps free up system resources by completing these tasks immediately, improving overall system responsiveness and optimizing resource allocation. It can also reduce latency caused by deferred operations in critical system processes.".to_string(),
        TweakCategory::Action,
        ProcessIdleTasksTweak,
        TweakWidget::Button,
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let binding = get_display_settings();
        let default = binding.last().unwrap();
        println!("Current display settings: {:?}", default);
        let result = set_display_settings(default.clone());
        assert_eq!(result, 0);
    }

    #[test]
    fn test_change_refresh_rate_to_30() {
        let result = set_display_settings(DisplaySettingsType {
            width: 3840,
            height: 2160,
            refresh_rate: 30,
        });
        assert_eq!(result, 0);
    }

    #[test]
    fn test_change_refresh_rate_to_60() {
        let result = set_display_settings(DisplaySettingsType {
            width: 3840,
            height: 2160,
            refresh_rate: 60,
        });
        assert_eq!(result, 0);
    }

    #[test]
    fn test_res_1024_768_60() {
        let result = set_display_settings(DisplaySettingsType {
            width: 1024,
            height: 768,
            refresh_rate: 60,
        });
        assert_eq!(result, 0);
    }

    #[test]
    fn test_4k_60() {
        let result = set_display_settings(DisplaySettingsType {
            width: 3840,
            height: 2160,
            refresh_rate: 60,
        });
        assert_eq!(result, 0);
    }

    #[test]
    fn test_tweak_apply() {
        let tweak = low_res_mode();
        let result = tweak.method.apply(TweakId::LowResMode);
        println!("{:?}", result);
    }

    #[test]
    fn test_tweak_revert() {
        let tweak = low_res_mode();
        let result = tweak.method.revert(TweakId::LowResMode);
        println!("{:?}", result);
    }
}

pub struct ProcessIdleTasksTweak;

impl TweakMethod for ProcessIdleTasksTweak {
    fn initial_state(&self, _id: TweakId) -> Result<bool, Error> {
        // Since this is an action, it doesn't have a state
        Ok(false)
    }

    fn apply(&self, id: TweakId) -> Result<(), Error> {
        tracing::info!("{:?} -> Running Process Idle Tasks.", id);

        let mut cmd = Command::new("Rundll32.exe");
        cmd.args(["advapi32.dll,ProcessIdleTasks"]);

        // Spawn the command asynchronously
        cmd.spawn().map_err(|e| {
            tracing::error!("{id:?} -> Failed to run ProcessIdleTasks: {e:?}");
            anyhow::Error::from(e)
        })?;

        // Return immediately without waiting for the command to finish
        Ok(())
    }

    fn revert(&self, _id: TweakId) -> Result<(), Error> {
        Ok(())
    }
}
