// src/tweaks/definitions/low_res_mode.rs

use std::{fs::File, io::Write, mem::zeroed, ptr::null_mut};

use anyhow::anyhow;
use tracing::info;
use winapi::{
    shared::minwindef::DWORD,
    um::{
        wingdi::{DEVMODEW, DM_DISPLAYFREQUENCY, DM_PELSHEIGHT, DM_PELSWIDTH},
        winuser::{
            ChangeDisplaySettingsW, EnumDisplaySettingsW, CDS_UPDATEREGISTRY,
            DISP_CHANGE_SUCCESSFUL, ENUM_CURRENT_SETTINGS,
        },
    },
};

use crate::tweaks::{TweakId, TweakMethod};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplaySettingsType {
    pub width: i32,
    pub height: i32,
    pub refresh_rate: i32,
}

/// Retrieves all available display settings using EnumDisplaySettingsW.
fn get_display_settings() -> Vec<DisplaySettingsType> {
    let mut settings = Vec::new();
    let mut dev_mode: DEVMODEW = unsafe { zeroed() };
    dev_mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

    let mut mode_num: DWORD = 0;

    loop {
        let result = unsafe { EnumDisplaySettingsW(null_mut(), mode_num, &mut dev_mode) };
        if result == 0 {
            break;
        }

        settings.push(DisplaySettingsType {
            width: dev_mode.dmPelsWidth as i32,
            height: dev_mode.dmPelsHeight as i32,
            refresh_rate: dev_mode.dmDisplayFrequency as i32,
        });

        mode_num += 1;
    }

    settings
}

/// Applies the specified display settings using ChangeDisplaySettingsW.
/// Returns the DISP_CHANGE result code.
fn set_display_settings(new_settings: DisplaySettingsType) -> i32 {
    let mut dev_mode: DEVMODEW = unsafe { zeroed() };
    dev_mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
    dev_mode.dmPelsWidth = new_settings.width as DWORD;
    dev_mode.dmPelsHeight = new_settings.height as DWORD;
    dev_mode.dmDisplayFrequency = new_settings.refresh_rate as DWORD;
    dev_mode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_DISPLAYFREQUENCY;

    
    unsafe { ChangeDisplaySettingsW(&mut dev_mode, CDS_UPDATEREGISTRY) }
}

/// Retrieves the current display settings using EnumDisplaySettingsW with ENUM_CURRENT_SETTINGS.
fn get_current_display_settings() -> Result<DisplaySettingsType, anyhow::Error> {
    let mut dev_mode: DEVMODEW = unsafe { zeroed() };
    dev_mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

    let result = unsafe { EnumDisplaySettingsW(null_mut(), ENUM_CURRENT_SETTINGS, &mut dev_mode) };
    if result == 0 {
        return Err(anyhow!("Failed to retrieve current display settings."));
    }

    Ok(DisplaySettingsType {
        width: dev_mode.dmPelsWidth as i32,
        height: dev_mode.dmPelsHeight as i32,
        refresh_rate: dev_mode.dmDisplayFrequency as i32,
    })
}

pub struct LowResMode {
    id: TweakId,
    pub default: DisplaySettingsType,
    pub target_state: DisplaySettingsType,
}

impl Default for LowResMode {
    fn default() -> Self {
        let options = get_display_settings();

        // Save all the options to a .tmp file
        let mut file = File::create("display_settings.tmp").expect("Unable to create temp file");
        for option in &options {
            writeln!(
                file,
                "{}x{} @{}hz",
                option.width, option.height, option.refresh_rate
            )
            .expect("Unable to write to temp file");
        }

        // Find the lowest refresh rate settings
        let min_refresh_rate = options
            .iter()
            .min_by_key(|x| x.refresh_rate)
            .expect("No display settings found");

        // Among those, find the lowest resolution (width)
        let min_resolution = options
            .iter()
            .filter(|x| x.refresh_rate == min_refresh_rate.refresh_rate)
            .min_by_key(|x| x.width)
            .expect("No matching display settings found");

        Self {
            id: TweakId::LowResMode,
            default: get_current_display_settings()
                .expect("Failed to get current display settings"),
            target_state: min_resolution.clone(),
        }
    }
}

impl TweakMethod for LowResMode {
    fn initial_state(&self) -> Result<bool, anyhow::Error> {
        let current = get_display_settings();
        let current_state = current.last().unwrap();
        info!(
            "{:?} -> Initial state: Current display settings: {:?}",
            self.id, current_state
        );
        Ok(current_state == &self.target_state)
    }

    fn apply(&self) -> Result<(), anyhow::Error> {
        let result = set_display_settings(self.target_state.clone());
        match result {
            DISP_CHANGE_SUCCESSFUL => Ok(()),
            code => Err(anyhow!(
                "{:?} -> Failed to apply display settings. Error code: {}",
                self.id,
                code
            )),
        }
    }

    fn revert(&self) -> Result<(), anyhow::Error> {
        let result = set_display_settings(self.default.clone());
        match result {
            DISP_CHANGE_SUCCESSFUL => Ok(()),
            code => Err(anyhow!(
                "{:?} -> Failed to revert display settings. Error code: {}",
                self.id,
                code
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use winapi::um::winuser::DISP_CHANGE_SUCCESSFUL;

    use super::*;

    #[test]
    fn test_default_state() {
        let settings = get_display_settings();
        let default = settings.last().unwrap();
        println!("Current display settings: {:?}", default);
        let result = set_display_settings(default.clone());
        assert_eq!(result, DISP_CHANGE_SUCCESSFUL);
    }

    #[test]
    fn test_change_refresh_rate_to_30() {
        let result = set_display_settings(DisplaySettingsType {
            width: 3840,
            height: 2160,
            refresh_rate: 30,
        });
        assert_eq!(result, DISP_CHANGE_SUCCESSFUL);
    }

    #[test]
    fn test_change_refresh_rate_to_60() {
        let result = set_display_settings(DisplaySettingsType {
            width: 3840,
            height: 2160,
            refresh_rate: 60,
        });
        assert_eq!(result, DISP_CHANGE_SUCCESSFUL);
    }

    #[test]
    fn test_res_1024_768_60() {
        let result = set_display_settings(DisplaySettingsType {
            width: 1024,
            height: 768,
            refresh_rate: 60,
        });
        assert_eq!(result, DISP_CHANGE_SUCCESSFUL);
    }

    #[test]
    fn test_4k_60() {
        let result = set_display_settings(DisplaySettingsType {
            width: 3840,
            height: 2160,
            refresh_rate: 60,
        });
        assert_eq!(result, DISP_CHANGE_SUCCESSFUL);
    }

    #[test]
    fn test_tweak_apply() {
        let tweak = LowResMode::default();
        let result = tweak.apply();
        println!("{:?}", result);
    }

    #[test]
    fn test_tweak_revert() {
        let tweak = LowResMode::default();
        let result = tweak.revert();
        println!("{:?}", result);
    }
}
