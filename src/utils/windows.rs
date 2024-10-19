// src/utils/windows.rs

use anyhow::Result as AnyResult;
use windows::{
    core::PWSTR as CorePWSTR,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
        System::{
            Threading::{GetCurrentProcess, OpenProcessToken},
            WindowsProgramming::GetUserNameW,
        },
    },
};

use super::powershell::execute_powershell_script;

/// Checks if the current process is running with elevated (administrator) privileges.
///
/// # Returns
///
/// - `true` if the process is elevated.
/// - `false` otherwise.
pub fn is_elevated() -> bool {
    let mut handle: HANDLE = HANDLE::default();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle).is_ok() } {
        let mut elevation: TOKEN_ELEVATION = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<TOKEN_ELEVATION>();
        let mut ret_size = size;
        if unsafe {
            GetTokenInformation(
                handle,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                size as u32,
                &mut ret_size as *mut _ as *mut _,
            )
            .is_ok()
        } {
            // Close the handle before returning
            if handle != HANDLE(std::ptr::null_mut()) && unsafe { CloseHandle(handle).is_err() } {
                return false;
            }
            return elevation.TokenIsElevated != 0;
        }
    }
    // Close the handle if it was opened
    if handle != HANDLE(std::ptr::null_mut()) && unsafe { CloseHandle(handle).is_err() } {
        return false;
    }
    false
}

/// Initiates a system reboot.
///
/// Returns:
/// - `Ok(())` if the reboot command was successfully executed.
/// - `Err(anyhow::Error)` if there was an error executing the reboot command.
pub fn reboot_system() -> AnyResult<()> {
    // For Windows, use the 'shutdown' command with '/r' flag to reboot
    execute_powershell_script("Restart-Computer -Force -Confirm:$false")?;

    Ok(())
}

/// Reboots the system into BIOS/UEFI settings.
/// Requires administrator privileges.
/// Note: This command works on Windows 10 and later.
pub fn reboot_into_bios() -> AnyResult<()> {
    execute_powershell_script("Restart-Computer -Force -Firmware")?;
    Ok(())
}

/// Retrieves the current username using the Windows API.
pub fn get_current_username() -> String {
    let mut buffer = [0u16; 256]; // Create a buffer for the username
    let mut size = buffer.len() as u32;

    let result = unsafe { GetUserNameW(CorePWSTR::from_raw(buffer.as_mut_ptr()), &mut size) };

    if result.is_ok() {
        String::from_utf16_lossy(&buffer[..size as usize - 1])
    } else {
        panic!("Failed to get current username")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_elevated() {
        let elevated = is_elevated();
        // This test should be run with appropriate privileges
        println!("Is elevated: {}", elevated);
    }
}
