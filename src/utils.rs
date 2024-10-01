// src/utils.rs

use std::process::Command;

use anyhow::{anyhow, Result as AnyResult};
use winapi::{
    shared::ntdef::HANDLE,
    um::{
        handleapi::CloseHandle,
        processthreadsapi::{GetCurrentProcess, OpenProcessToken},
        securitybaseapi::GetTokenInformation,
        winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
    },
};

/// Checks if the current process is running with elevated (administrator) privileges.
///
/// # Returns
///
/// - `true` if the process is elevated.
/// - `false` otherwise.
pub fn is_elevated() -> bool {
    let mut handle: HANDLE = std::ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) } != 0 {
        let mut elevation: TOKEN_ELEVATION = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<TOKEN_ELEVATION>();
        let mut ret_size = size;
        if unsafe {
            GetTokenInformation(
                handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size as u32,
                &mut ret_size as *mut _ as *mut _,
            ) != 0
        } {
            // Close the handle before returning
            if !handle.is_null() {
                unsafe { CloseHandle(handle) };
            }
            return elevation.TokenIsElevated != 0;
        }
    }
    // Close the handle if it was opened
    if !handle.is_null() {
        unsafe { CloseHandle(handle) };
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
    #[cfg(target_os = "windows")]
    {
        Command::new("shutdown")
            .args(["/r", "/t", "0"])
            .status()
            .map_err(|e| anyhow!("Failed to execute shutdown command: {}", e))?;
    }

    Ok(())
}

/// Reboots the system into BIOS/UEFI settings.
/// Requires administrator privileges.
/// Note: This command works on Windows 10 and later.
pub fn reboot_into_bios() -> AnyResult<()> {
    Command::new("shutdown")
        .args(["/r", "/fw", "/t", "0"])
        .status()
        .map(|_| ())
        .map_err(|e| anyhow!("Failed to execute shutdown into BIOS command: {}", e))
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
