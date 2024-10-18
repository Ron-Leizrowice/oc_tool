// src/power.rs

use std::ptr;

use anyhow::{anyhow, Result};
use widestring::U16String;
use windows::{
    core::{GUID, HRESULT},
    Win32::{
        Foundation::{LocalFree, ERROR_NO_MORE_ITEMS, HLOCAL, WIN32_ERROR},
        System::Power::{
            PowerDuplicateScheme, PowerEnumerate, PowerGetActiveScheme, PowerReadACValueIndex,
            PowerReadFriendlyName, PowerRestoreDefaultPowerSchemes, PowerSetActiveScheme,
            PowerWriteACValueIndex, ACCESS_SCHEME, POWER_DATA_ACCESSOR,
        },
    },
};

#[derive(Debug)]
pub struct PowerScheme {
    pub name: String,
    pub guid: GUID,
}

#[derive(Debug)]
pub struct PowerSubgroup {
    pub _name: String,
    pub _guid: GUID,
    pub settings: Vec<PowerSetting>, // Added `settings` field
}

#[derive(Debug)]
pub struct PowerSetting {
    pub name: String,
    pub guid: GUID,
}

/// Enumerates all power schemes on the system.
pub fn get_all_power_schemes() -> Result<Vec<PowerScheme>, windows::core::Error> {
    let mut schemes = Vec::new();
    let mut index = 0;

    loop {
        match power_enumerate(ACCESS_SCHEME, None, None, index) {
            Ok(guid) => {
                // Retrieve the friendly name for the scheme
                let name = match read_friendly_name(&guid, None, None) {
                    Ok(name) => name,
                    Err(_) => format!("Unknown Scheme {}", index),
                };

                schemes.push(PowerScheme { name, guid });
                index += 1;
            }
            Err(e) => {
                if e.code().0 as u32 == ERROR_NO_MORE_ITEMS.0
                    || e.message() == "The operation completed successfully."
                {
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }

    if schemes.is_empty() {
        Err(windows::core::Error::new(
            HRESULT(1168),
            "No power schemes found",
        ))
    } else {
        Ok(schemes)
    }
}

/// Enumerates all power subgroups and their settings within a given power scheme.
pub fn enumerate_power_subgroups_and_settings(
    scheme_guid: &GUID,
) -> Result<Vec<PowerSubgroup>, windows::core::Error> {
    let mut subgroups = Vec::new();
    let mut index = 0;

    loop {
        match power_enumerate(ACCESS_SCHEME, Some(scheme_guid), None, index) {
            Ok(subgroup_guid) => {
                // Retrieve the friendly name for the subgroup
                let subgroup_name =
                    match read_friendly_name(scheme_guid, Some(&subgroup_guid), None) {
                        Ok(name) => name,
                        Err(_) => format!("Unknown Subgroup {}", index),
                    };

                // Enumerate power settings within this subgroup
                let settings =
                    enumerate_power_settings_within_subgroup(scheme_guid, &subgroup_guid)?;

                subgroups.push(PowerSubgroup {
                    _name: subgroup_name,
                    _guid: subgroup_guid,
                    settings,
                });

                index += 1;
            }
            Err(e) => {
                if e.code().0 as u32 == ERROR_NO_MORE_ITEMS.0
                    || e.message() == "The operation completed successfully."
                {
                    break;
                } else {
                    tracing::error!("Error: {:?}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(subgroups)
}

/// Enumerates all power settings within a given subgroup.
fn enumerate_power_settings_within_subgroup(
    scheme_guid: &GUID,
    subgroup_guid: &GUID,
) -> Result<Vec<PowerSetting>, windows::core::Error> {
    let mut settings = Vec::new();
    let mut index = 0;

    loop {
        match power_enumerate(ACCESS_SCHEME, Some(scheme_guid), Some(subgroup_guid), index) {
            Ok(setting_guid) => {
                // Retrieve the friendly name for the setting
                let setting_name =
                    match read_friendly_name(scheme_guid, Some(subgroup_guid), Some(&setting_guid))
                    {
                        Ok(name) => name,
                        Err(_) => format!("Unknown Setting {}", index),
                    };

                settings.push(PowerSetting {
                    name: setting_name,
                    guid: setting_guid,
                });

                index += 1;
            }
            Err(e) => {
                if e.code().0 as u32 == ERROR_NO_MORE_ITEMS.0
                    || e.message() == "The operation completed successfully."
                {
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }

    Ok(settings)
}

/// Retrieves the friendly name for a given power scheme, subgroup, or setting.
/// If `setting_guid` is `None`, it retrieves the name of the subgroup.
/// If both `subgroup_guid` and `setting_guid` are `None`, it retrieves the name of the scheme.
fn read_friendly_name(
    scheme_guid: &GUID,
    subgroup_guid: Option<&GUID>,
    setting_guid: Option<&GUID>,
) -> Result<String, windows::core::Error> {
    let mut buffer_size: u32 = 0;

    // First call to determine the required buffer size
    let result = unsafe {
        PowerReadFriendlyName(
            None,              // RootPowerKey
            Some(scheme_guid), // SchemeGuid
            subgroup_guid.map(|g| g as *const GUID),
            setting_guid.map(|g| g as *const GUID),
            None, // Buffer
            &mut buffer_size,
        )
    };

    // Handle specific error codes
    if result == WIN32_ERROR(0) || result == WIN32_ERROR(234) {
        // Proceed to allocate buffer
    } else if result == WIN32_ERROR(2) {
        // ERROR_FILE_NOT_FOUND: No friendly name available
        return Ok("Unknown".to_string());
    } else {
        // Other errors
        return Err(windows::core::Error::from_win32());
    }

    // Allocate a buffer with the required size (buffer_size is in bytes)
    // Ensure buffer_size is at least 2 bytes to hold an empty string
    if buffer_size < 2 {
        buffer_size = 2;
    }
    let mut buffer: Vec<u16> = vec![0; (buffer_size / 2) as usize]; // u16 for wide strings

    // Second call to actually get the friendly name
    let result = unsafe {
        PowerReadFriendlyName(
            None,              // RootPowerKey
            Some(scheme_guid), // SchemeGuid
            subgroup_guid.map(|g| g as *const GUID),
            setting_guid.map(|g| g as *const GUID),
            Some(buffer.as_mut_ptr() as *mut u8), // Buffer
            &mut buffer_size,                     // BufferSize
        )
    };

    if result == WIN32_ERROR(0) {
        // Successfully retrieved the friendly name
    } else {
        return Err(windows::core::Error::from_win32());
    }

    // Find the null terminator to determine the actual string length
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());

    buffer.truncate(len);

    // Convert the wide string to a Rust `String`
    let friendly_name = U16String::from_vec(buffer).to_string_lossy();

    // If the friendly name is empty, default to "Unknown"
    if friendly_name.is_empty() {
        Ok("Unknown".to_string())
    } else {
        Ok(friendly_name)
    }
}

/// Enumerates the specified elements in a power scheme.
///
/// This function is typically called in a loop, incrementing the `index` parameter to retrieve
/// subkeys until all elements have been enumerated.
///
/// # Parameters
///
/// - `access_flags`: A set of flags that specifies what will be enumerated.
///   - `ACCESS_SCHEME` (16): Enumerate power schemes. The `SchemeGuid` and `SubGroupOfPowerSettingsGuid` parameters are ignored.
///   - `ACCESS_SUBGROUP` (17): Enumerate subgroups under `SchemeGuid`. The `SubGroupOfPowerSettingsGuid` parameter is ignored.
///   - `ACCESS_INDIVIDUAL_SETTING` (18): Enumerate individual power settings under `SchemeGuid\SubGroupOfPowerSettingsGuid`.
///
/// - `scheme_guid`: The identifier of the power scheme. If `None`, an enumeration of the power policies is returned.
///
/// - `subgroup_guid`: The identifier of the subgroup of power settings. If `None`, an enumeration of settings under the `SchemeGuid` key is returned.
///
/// - `index`: The zero-based index of the scheme, subgroup, or setting that is being enumerated.
///
/// # Returns
///
/// - `Ok(GUID)`: The `GUID` of the enumerated element.
/// - `Err(windows::core::Error)`: An error if the enumeration fails.
///
/// # Errors
///
/// This function may return errors corresponding to various Win32 error codes, including but not limited to:
///
/// - `ERROR_INVALID_PARAMETER` (87): One of the parameters is not valid.
/// - `ERROR_MORE_DATA` (234): The buffer size is insufficient. This should not occur as the buffer is sized appropriately.
/// - `ERROR_NO_MORE_ITEMS` (259): No more items are available to enumerate.
/// - Other errors as returned by the `PowerEnumerate` API.
fn power_enumerate(
    access_flags: POWER_DATA_ACCESSOR,
    scheme_guid: Option<&GUID>,
    subgroup_guid: Option<&GUID>,
    index: u32,
) -> Result<GUID, windows::core::Error> {
    // Initialize buffer size
    let mut buffer_size: u32 = 0;

    // First call to determine the required buffer size
    let result = unsafe {
        PowerEnumerate(
            None, // RootPowerKey (must be NULL)
            scheme_guid.map(|guid| guid as *const GUID),
            subgroup_guid.map(|guid| guid as *const GUID),
            access_flags,
            index,
            None, // Buffer
            &mut buffer_size,
        )
    };

    // Check if the first call succeeded or returned ERROR_MORE_DATA
    if result != WIN32_ERROR(0) && result != WIN32_ERROR(234) {
        // 234 is ERROR_MORE_DATA
        return Err(windows::core::Error::from_win32());
    }

    // Allocate a buffer to receive the GUID
    // GUID is 16 bytes, ensure buffer_size is at least 16
    if buffer_size < 16 {
        buffer_size = 16;
    }

    // Create a buffer for the GUID
    let mut buffer: Vec<u8> = vec![0; buffer_size as usize];

    // Second call to actually get the GUID
    let result = unsafe {
        PowerEnumerate(
            None, // RootPowerKey (must be NULL)
            scheme_guid.map(|guid| guid as *const GUID),
            subgroup_guid.map(|guid| guid as *const GUID),
            access_flags,
            index,
            Some(buffer.as_mut_ptr()),
            &mut buffer_size,
        )
    };

    // Check if the second call was successful
    if result != WIN32_ERROR(0) {
        return Err(windows::core::Error::from_win32());
    }

    // Ensure that the buffer contains a valid GUID
    if buffer_size < 16 {
        return Err(windows::core::Error::from_win32());
    }

    // Correctly construct the GUID by handling endianness
    let guid = GUID {
        data1: u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
        data2: u16::from_le_bytes([buffer[4], buffer[5]]),
        data3: u16::from_le_bytes([buffer[6], buffer[7]]),
        data4: [
            buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
            buffer[15],
        ],
    };

    Ok(guid)
}

/// Sets the active power scheme for the current user.
///
/// This function changes the active power scheme to the one identified by `scheme_guid`.
///
/// # Parameters
///
/// - `scheme_guid`: A reference to the GUID of the power scheme to set as active.
///
/// # Returns
///
/// - `Ok(())`: The power scheme was successfully set as active.
/// - `Err(windows::core::Error)`: An error occurred while attempting to set the active power scheme.
///
/// # Errors
///
/// This function may return errors corresponding to various Win32 error codes, including but not limited to:
///
/// - `ERROR_INVALID_PARAMETER` (87): One of the parameters is not valid.
/// - `ERROR_ACCESS_DENIED` (5): The user does not have the necessary permissions to set the power scheme.
/// - `ERROR_NOT_FOUND` (1168): The specified power scheme GUID does not correspond to any existing power scheme.
/// - Other errors as returned by the `PowerSetActiveScheme` API.
pub fn set_active_power_scheme(scheme_guid: &GUID) -> Result<(), windows::core::Error> {
    // Call PowerSetActiveScheme to set the active power scheme
    let result = unsafe {
        PowerSetActiveScheme(
            None,              // UserRootPowerKey (must be NULL)
            Some(scheme_guid), // SchemeGuid
        )
    };

    // Check if the API call was successful
    if result != WIN32_ERROR(0) {
        return Err(windows::core::Error::from_win32());
    }

    Ok(())
}

/// Sets the value for a power setting.
pub fn write_ac_value_index(
    scheme_guid: &GUID,
    subgroup_guid: &GUID,
    power_setting_guid: &GUID,
    value: u32,
) -> Result<(), anyhow::Error> {
    unsafe {
        let hr = PowerWriteACValueIndex(
            None, // RootPowerKey
            scheme_guid as *const GUID,
            Some(subgroup_guid as *const GUID),
            Some(power_setting_guid as *const GUID),
            value,
        );

        if hr == WIN32_ERROR(0) {
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to write power setting: HRESULT(0x{:08X})",
                hr.0
            ))
        }
    }
}

/// Reads the AC value index for a power setting.
pub fn read_ac_value_index(
    scheme_guid: &GUID,
    subgroup_guid: &GUID,
    power_setting_guid: &GUID,
) -> Result<u32, anyhow::Error> {
    let mut value: u32 = 0;
    unsafe {
        // Call the PowerReadACValueIndex API
        let hr = PowerReadACValueIndex(
            None, // RootPowerKey
            Some(scheme_guid as *const GUID),
            Some(subgroup_guid as *const GUID),
            Some(power_setting_guid as *const GUID),
            &mut value,
        );

        // Check if the operation was successful
        if hr == WIN32_ERROR(0) {
            Ok(value)
        } else {
            Err(anyhow!(
                "Failed to read power setting: HRESULT(0x{:08X})",
                hr.0
            ))
        }
    }
}

/// Retrieves the currently active power scheme.
pub fn get_active_power_scheme() -> Result<PowerScheme, windows::core::Error> {
    let active_scheme_guid = read_active_scheme_guid()?;
    let active_scheme_name = read_friendly_name(&active_scheme_guid, None, None)?;

    Ok(PowerScheme {
        name: active_scheme_name,
        guid: active_scheme_guid,
    })
}

/// Reads the active power scheme GUID.
fn read_active_scheme_guid() -> Result<GUID, windows::core::Error> {
    let mut guid_pointer: *mut GUID = ptr::null_mut();

    // Call PowerGetActiveScheme to get the active power scheme GUID
    let result = unsafe {
        PowerGetActiveScheme(
            None,              // UserRootPowerKey
            &mut guid_pointer, // ActivePolicyGuid
        )
    };

    // Check if the API call was successful
    if result != WIN32_ERROR(0) {
        return Err(windows::core::Error::from_win32());
    }

    if guid_pointer.is_null() {
        return Err(windows::core::Error::from_win32());
    }

    // Wrap the raw pointer in a smart wrapper to ensure it gets freed
    let active_scheme = PowerSchemeGuid { ptr: guid_pointer };

    // Dereference the pointer to get the GUID value
    let active_scheme_guid = unsafe { *active_scheme.ptr };

    // `active_scheme` is dropped here, automatically calling `LocalFree`

    Ok(active_scheme_guid)
}

/// Smart wrapper for GUID pointer to ensure automatic memory freeing.
struct PowerSchemeGuid {
    ptr: *mut GUID,
}

impl Drop for PowerSchemeGuid {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let hclocal = HLOCAL(self.ptr as *mut _);
                let free_result = LocalFree(hclocal);
                if !free_result.is_invalid() {
                    panic!("Failed to free memory for PowerSchemeGuid.");
                }
            }
        }
    }
}

/// Duplicates an existing power scheme.
pub fn duplicate_power_scheme(source_scheme_guid: &GUID) -> Result<GUID, windows::core::Error> {
    // Initialize a null pointer for the destination GUID
    let mut destination_guid_ptr: *mut GUID = ptr::null_mut();

    // Call PowerDuplicateScheme to duplicate the power scheme
    let result = unsafe {
        PowerDuplicateScheme(
            None,                      // RootPowerKey (must be NULL)
            source_scheme_guid,        // SourceSchemeGuid
            &mut destination_guid_ptr, // DestinationSchemeGuid
        )
    };

    // Check if the API call was successful
    if result != WIN32_ERROR(0) {
        return Err(windows::core::Error::from_win32());
    }

    if destination_guid_ptr.is_null() {
        return Err(windows::core::Error::from_win32());
    }

    let destination_scheme = PowerSchemeGuid {
        ptr: destination_guid_ptr,
    };

    // Dereference the pointer to get the GUID value
    let duplicated_scheme_guid = unsafe { *destination_scheme.ptr };

    // `destination_scheme` is dropped here, automatically calling `LocalFree`

    Ok(duplicated_scheme_guid)
}

#[allow(dead_code)]
pub fn restore_default_power_schemes() -> Result<(), windows::core::Error> {
    let result = unsafe { PowerRestoreDefaultPowerSchemes() };

    if result != WIN32_ERROR(0) {
        return Err(windows::core::Error::from_win32());
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn set_balanced_power_scheme() {
        let balanced_guid = GUID::from_u128(0x381b4222_f694_41f0_9685_ff5bb260df2e);
        let result = set_active_power_scheme(&balanced_guid);
        assert!(result.is_ok());
    }

    #[test]
    fn set_high_performance_power_scheme() {
        let high_performance_guid = GUID::from_u128(0x8c5e7fda_e8bf_4a96_9a85_a6e23a8c635c);
        let result = set_active_power_scheme(&high_performance_guid);
        assert!(result.is_ok());
    }

    #[test]
    fn test_restore_default_power_schemes() {
        restore_default_power_schemes().unwrap();
    }
}
