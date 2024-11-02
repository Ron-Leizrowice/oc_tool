// src/tweaks/definitions/kill_explorer.rs

use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use anyhow::Error;
use tracing::{error, info};
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::CloseHandle,
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                CreateProcessW, OpenProcess, TerminateProcess, PROCESS_CREATION_FLAGS,
                PROCESS_INFORMATION, PROCESS_TERMINATE, STARTUPINFOW,
            },
        },
    },
};
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey,
};

use crate::tweaks::{TweakId, TweakMethod, TweakOption};

/// Struct implementing the TweakMethod trait for killing Explorer.
pub struct KillExplorerTweak {
    pub id: TweakId,
}

impl KillExplorerTweak {
    /// Creates a new instance of the KillExplorerTweak struct.
    pub fn new() -> Self {
        Self {
            id: TweakId::KillExplorer,
        }
    }
}

impl TweakMethod for KillExplorerTweak {
    /// Checks the initial state of the tweak.
    fn initial_state(&self) -> Result<TweakOption, Error> {
        let explorer_running = is_explorer_running()?;
        let auto_restart_enabled = is_auto_restart_enabled()?;

        // Tweak is considered enabled if explorer is not running and auto-restart is disabled.
        if !explorer_running && !auto_restart_enabled {
            info!("{:?} -> Initial state: Enabled", self.id);
            Ok(TweakOption::Enabled(true))
        } else {
            info!("{:?} -> Initial state: Disabled", self.id);
            Ok(TweakOption::Enabled(false))
        }
    }

    /// Applies the tweak: Terminates Explorer and prevents it from restarting.
    fn apply(&self, _option: TweakOption) -> Result<(), Error> {
        info!(
            "{:?} -> Terminating Explorer process and preventing restart...",
            self.id
        );

        // Terminate all explorer.exe processes.
        kill_explorer_processes()?;

        // Disable automatic restart of Explorer.
        set_auto_restart_shell(false)?;

        info!("Explorer has been terminated and prevented from restarting.");

        Ok(())
    }

    /// Reverts the tweak: Allows Explorer to restart and starts it.
    fn revert(&self) -> Result<(), Error> {
        info!(
            "{:?} -> Allowing Explorer to restart and starting it...",
            self.id
        );

        // Enable automatic restart of Explorer.
        set_auto_restart_shell(true)?;

        // Start explorer.exe.
        start_explorer()?;

        info!("Explorer has been allowed to restart and has been started.");

        Ok(())
    }
}

/// Checks if any instances of explorer.exe are currently running.
fn is_explorer_running() -> Result<bool, Error> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut found = false;

    if unsafe { Process32FirstW(snapshot, &mut entry).is_ok() } {
        loop {
            // Convert the wide string to a Rust String, trimming null terminators.
            let exe_name_utf16: Vec<u16> = entry
                .szExeFile
                .iter()
                .take_while(|&&c| c != 0)
                .cloned()
                .collect();
            let exe_name = String::from_utf16_lossy(&exe_name_utf16);

            if exe_name.eq_ignore_ascii_case("explorer.exe") {
                found = true;
                break;
            }

            if !unsafe { Process32NextW(snapshot, &mut entry).is_ok() } {
                break;
            }
        }
    }

    if let Err(e) = unsafe { CloseHandle(snapshot) } {
        error!("Failed to close snapshot handle: {:?}", e);
    }

    Ok(found)
}

/// Checks if the AutoRestartShell registry key is enabled.
fn is_auto_restart_enabled() -> Result<bool, Error> {
    // Access the HKEY_LOCAL_MACHINE hive.
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon";
    let key = hklm.open_subkey_with_flags(key_path, KEY_READ)?;

    // Retrieve the value of AutoRestartShell.
    let auto_restart_shell: Result<u32, _> = key.get_value("AutoRestartShell");
    match auto_restart_shell {
        Ok(value) => Ok(value == 1),
        Err(_) => Ok(false), // Default to false if the key is not set.
    }
}

/// Sets the AutoRestartShell registry key.
///
/// - `enable`: If `true`, sets AutoRestartShell to 1. Otherwise, sets it to 0.
fn set_auto_restart_shell(enable: bool) -> Result<(), Error> {
    // Access the HKEY_LOCAL_MACHINE hive.
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon";
    let (key, _) = hklm.create_subkey_with_flags(key_path, KEY_WRITE)?;

    // Set the value of AutoRestartShell.
    key.set_value("AutoRestartShell", &if enable { 1u32 } else { 0u32 })?;

    Ok(())
}

/// Terminates all running instances of explorer.exe.
fn kill_explorer_processes() -> Result<(), Error> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    if unsafe { Process32FirstW(snapshot, &mut entry).is_ok() } {
        loop {
            // Convert the wide string to a Rust String, trimming null terminators.
            let exe_name_utf16: Vec<u16> = entry
                .szExeFile
                .iter()
                .take_while(|&&c| c != 0)
                .cloned()
                .collect();
            let exe_name = String::from_utf16_lossy(&exe_name_utf16);

            if exe_name.eq_ignore_ascii_case("explorer.exe") {
                // Open the process with termination rights.
                let process_handle =
                    unsafe { OpenProcess(PROCESS_TERMINATE, false, entry.th32ProcessID)? };

                if !process_handle.is_invalid() {
                    // Terminate the process.
                    let success = unsafe { TerminateProcess(process_handle, 0).is_ok() };
                    if success {
                        info!("Terminated explorer.exe with PID {}", entry.th32ProcessID);
                    } else {
                        error!(
                            "Failed to terminate explorer.exe with PID {}",
                            entry.th32ProcessID
                        );
                    }

                    // Close the process handle.
                    if let Err(e) = unsafe { CloseHandle(process_handle) } {
                        error!("Failed to close process handle: {:?}", e);
                    }
                } else {
                    error!(
                        "Failed to open explorer.exe with PID {}",
                        entry.th32ProcessID
                    );
                }
            }

            if !unsafe { Process32NextW(snapshot, &mut entry).is_ok() } {
                break;
            }
        }
    }

    if let Err(e) = unsafe { CloseHandle(snapshot) } {
        error!("Failed to close snapshot handle: {:?}", e);
    }

    Ok(())
}

/// Starts the explorer.exe process.
fn start_explorer() -> Result<(), Error> {
    let application_name = "C:\\Windows\\explorer.exe";
    let application_name_wide: Vec<u16> = OsStr::new(application_name)
        .encode_wide()
        .chain(std::iter::once(0)) // Null terminator.
        .collect();

    let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
    startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    // Create the process.
    let result = unsafe {
        CreateProcessW(
            PCWSTR(application_name_wide.as_ptr()),
            PWSTR(std::ptr::null_mut()), // lpCommandLine is optional when lpApplicationName is provided.
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS(0),
            None,
            None,
            &startup_info,
            &mut process_info,
        )
    };

    if result.is_err() {
        return Err(Error::msg("Failed to start explorer.exe"));
    }

    // Close process and thread handles to prevent handle leaks.
    unsafe {
        if CloseHandle(process_info.hProcess).is_err() {
            if let Err(e) = CloseHandle(process_info.hThread) {
                error!("Failed to close thread handle: {:?}", e);
            }
        }
        if let Err(e) = CloseHandle(process_info.hThread) {
            error!("Failed to close thread handle: {:?}", e);
        }
    }

    Ok(())
}
