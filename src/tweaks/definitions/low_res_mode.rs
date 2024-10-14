// src/tweaks/definitions/low_res_mode.rs

use DisplaySettings::{get_display_settings, set_display_settings, DisplaySettingsType};

use crate::tweaks::{TweakId, TweakMethod};

pub struct LowResMode {
    id: TweakId,
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
            id: TweakId::LowResMode,
            default: get_display_settings().last().unwrap().clone(),
            target_state,
        }
    }
}

impl TweakMethod for LowResMode {
    fn initial_state(&self) -> Result<bool, anyhow::Error> {
        let binding = get_display_settings();
        let current = binding.last().unwrap();
        tracing::info!("{:?} -> Current display settings: {:?}", self.id, current);
        Ok(current == &self.target_state)
    }

    fn apply(&self) -> Result<(), anyhow::Error> {
        let result = set_display_settings(self.target_state.clone());
        match result {
            0 => Ok(()),
            _ => Err(anyhow::anyhow!(
                "{:?} -> Failed to apply display settings. Error code: {}",
                self.id,
                result
            )),
        }
    }

    fn revert(&self) -> Result<(), anyhow::Error> {
        let result = set_display_settings(self.default.clone());
        match result {
            0 => Ok(()),
            _ => Err(anyhow::anyhow!(
                "{:?} -> Failed to revert display settings. Error code: {}",
                self.id,
                result
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use DisplaySettings::{get_display_settings, set_display_settings, DisplaySettingsType};

    use crate::tweaks::rust::low_res_mode;

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
        let result = tweak.method.apply();
        println!("{:?}", result);
    }

    #[test]
    fn test_tweak_revert() {
        let tweak = low_res_mode();
        let result = tweak.method.revert();
        println!("{:?}", result);
    }
}
