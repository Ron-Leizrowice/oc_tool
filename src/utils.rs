// src/utils.rs

use std::{process::Command, ptr::null_mut};

use winapi::um::{
    handleapi::CloseHandle,
    processthreadsapi::{GetCurrentProcess, OpenProcessToken},
    securitybaseapi::GetTokenInformation,
    winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY},
};

/// Checks if the current process is running with elevated (administrator) privileges.
///
/// # Returns
///
/// - `true` if the process is elevated.
/// - `false` otherwise.
pub fn is_elevated() -> bool {
    let mut handle: HANDLE = null_mut();
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
pub fn reboot_system() -> Result<(), anyhow::Error> {
    // For Windows, use the 'shutdown' command with '/r' flag to reboot
    #[cfg(target_os = "windows")]
    {
        Command::new("shutdown")
            .args(["/r", "/t", "0"])
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute shutdown command: {}", e))?;
    }

    Ok(())
}
