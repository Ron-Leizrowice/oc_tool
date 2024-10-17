// src/tweaks/powershell.rs

use std::ffi::CString;

use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};
use windows::{
    core::PSTR,
    Win32::{
        Foundation::{
            CloseHandle, SetHandleInformation, HANDLE, HANDLE_FLAGS, HANDLE_FLAG_INHERIT, TRUE,
            WAIT_OBJECT_0,
        },
        Security::SECURITY_ATTRIBUTES,
        Storage::FileSystem::ReadFile,
        System::{
            Pipes::CreatePipe,
            Threading::{
                CreateProcessA, WaitForSingleObject, CREATE_NO_WINDOW, INFINITE,
                PROCESS_INFORMATION, STARTF_USESHOWWINDOW, STARTF_USESTDHANDLES, STARTUPINFOA,
            },
        },
        UI::WindowsAndMessaging::SW_HIDE,
    },
};

use super::{definitions::TweakId, TweakMethod};

/// Represents a PowerShell-based tweak, including scripts to read, apply, and undo the tweak.
#[derive(Clone, Debug)]
pub struct PowershellTweak {
    /// The unique ID of the tweak
    pub id: TweakId,
    /// PowerShell script to read the current state of the tweak.
    pub read_script: Option<String>,
    /// PowerShell script to apply the tweak.
    pub apply_script: String,
    /// PowerShell script to undo the tweak.
    pub undo_script: Option<String>,
    /// The target state of the tweak (e.g., the expected output of the read script when the tweak is enabled).
    pub target_state: Option<String>,
}

impl PowershellTweak {
    /// Reads the current state of the tweak by executing the `read_script`.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` with the current state if `read_script` is defined and succeeds.
    /// - `Ok(None)` if no `read_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn read_current_state(&self) -> Result<Option<String>> {
        if let Some(script) = &self.read_script {
            info!(
                "{:?} -> Reading current state of PowerShell tweak.",
                self.id
            );

            // Execute the PowerShell script using the custom function
            let output = execute_powershell_script(script).with_context(|| {
                format!(
                    "{:?} -> Failed to execute read PowerShell script '{}'",
                    self.id, script
                )
            })?;

            debug!(
                "{:?} -> PowerShell script output: {}",
                self.id,
                output.trim()
            );

            Ok(Some(output.trim().to_string()))
        } else {
            debug!(
                "{:?} -> No read script defined for PowerShell tweak. Skipping read operation.",
                self.id
            );
            Ok(None)
        }
    }
}

impl TweakMethod for PowershellTweak {
    /// Checks if the tweak is currently enabled by comparing the current value to the default value.
    /// If the current value matches the default value, the tweak is considered enabled.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation succeeds and the tweak is enabled.
    /// - `Ok(false)` if the operation succeeds and the tweak is disabled.
    /// - `Err(anyhow::Error)` if the operation fails.
    fn initial_state(&self) -> Result<bool> {
        if let Some(target_state) = &self.target_state {
            info!("{:?} -> Checking if PowerShell tweak is enabled.", self.id);
            match self.read_current_state() {
                Ok(Some(current_state)) => {
                    // check if the target state string is contained in the current state
                    let is_enabled = current_state.contains(target_state);
                    debug!(
                        "{:?} -> Current state: '{}', Target state: '{}', Enabled: {}",
                        self.id, current_state, target_state, is_enabled
                    );
                    Ok(is_enabled)
                }
                Ok(None) => {
                    warn!(
                        "{:?} -> No read script defined for PowerShell tweak. Assuming disabled.",
                        self.id
                    );
                    Ok(false)
                }
                Err(e) => {
                    error!(
                        error = ?e,
                        "{:?} -> Failed to read current state of PowerShell tweak.", self.id
                    );
                    Err(e)
                }
            }
        } else {
            warn!(
                "{:?} -> No target state defined for PowerShell tweak. Assuming disabled.",
                self.id
            );
            Ok(false)
        }
    }

    /// Executes the `apply_script` to apply the tweak synchronously.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn apply(&self) -> Result<()> {
        info!(
            "{:?} -> Applying PowerShell tweak using script '{}'.",
            self.id, &self.apply_script
        );

        // Execute the PowerShell script using the custom function
        let output = execute_powershell_script(&self.apply_script).with_context(|| {
            format!(
                "{:?} -> Failed to execute apply PowerShell script '{}'",
                self.id, &self.apply_script
            )
        })?;

        debug!(
            "{:?} -> Apply script executed successfully. Output: {}",
            self.id,
            output.trim()
        );
        Ok(())
    }

    /// Executes the `undo_script` to revert the tweak synchronously.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the script executes successfully or no `undo_script` is defined.
    /// - `Err(anyhow::Error)` if the script execution fails.
    fn revert(&self) -> Result<()> {
        if let Some(script) = &self.undo_script {
            info!(
                "{:?} -> Reverting PowerShell tweak using script '{}'.",
                self.id, script
            );

            // Execute the PowerShell script using the custom function
            let output = execute_powershell_script(script).with_context(|| {
                format!(
                    "{:?} -> Failed to execute revert PowerShell script '{}'",
                    self.id, script
                )
            })?;

            debug!(
                "{:?} -> Revert script executed successfully. Output: {}",
                self.id,
                output.trim()
            );
            Ok(())
        } else {
            warn!(
                "{:?} -> No undo script defined for PowerShell tweak. Skipping revert operation.",
                self.id
            );
            Ok(())
        }
    }
}

/// Executes a PowerShell script using Windows APIs and captures stdout and stderr separately.
///
/// # Arguments
///
/// * `script` - The PowerShell script to execute.
///
/// # Returns
///
/// * `Ok((stdout, stderr))` containing the standard output and standard error.
/// * `Err(anyhow::Error)` if the script execution fails.
pub fn execute_powershell_script(script: &str) -> Result<String> {
    // Step 1: Create security attributes to allow handle inheritance
    let mut sa = SECURITY_ATTRIBUTES::default();
    sa.nLength = std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
    sa.bInheritHandle = TRUE;
    sa.lpSecurityDescriptor = std::ptr::null_mut();

    // Step 2: Create pipes for stdout and stderr
    let mut stdout_read: HANDLE = HANDLE(std::ptr::null_mut());
    let mut stdout_write: HANDLE = HANDLE(std::ptr::null_mut());
    let mut stderr_read: HANDLE = HANDLE(std::ptr::null_mut());
    let mut stderr_write: HANDLE = HANDLE(std::ptr::null_mut());

    unsafe {
        // Create stdout pipe
        CreatePipe(&mut stdout_read, &mut stdout_write, Some(&sa), 0)
            .ok()
            .context("Failed to create stdout pipe")?;

        // Ensure the read handle is not inherited
        SetHandleInformation(stdout_read, HANDLE_FLAG_INHERIT.0 as u32, HANDLE_FLAGS(0))
            .ok()
            .context("Failed to set handle information for stdout_read")?;

        // Create stderr pipe
        CreatePipe(&mut stderr_read, &mut stderr_write, Some(&sa), 0)
            .ok()
            .context("Failed to create stderr pipe")?;

        // Ensure the read handle is not inherited
        SetHandleInformation(stderr_read, HANDLE_FLAG_INHERIT.0 as u32, HANDLE_FLAGS(0))
            .ok()
            .context("Failed to set handle information for stderr_read")?;
    }

    // Step 3: Set up the STARTUPINFOA structure
    let mut startup_info = STARTUPINFOA::default();
    startup_info.cb = std::mem::size_of::<STARTUPINFOA>() as u32;
    startup_info.dwFlags |= STARTF_USESHOWWINDOW | STARTF_USESTDHANDLES;
    startup_info.wShowWindow = SW_HIDE.0 as u16;
    startup_info.hStdOutput = stdout_write;
    startup_info.hStdError = stderr_write;
    // Optionally, you can redirect stdin if needed:
    // startup_info.hStdInput = stdin_read;

    // Step 4: Prepare the command line
    let command_line_str = format!(
        "powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command \"{}\"",
        script.replace('"', r#"\""#) // Escape double quotes in the script
    );

    // Convert to CString to ensure it's null-terminated
    let command_line_cstr = CString::new(command_line_str)
        .with_context(|| "Failed to convert command line to CString")?;

    // Obtain a mutable pointer and cast it to *mut u8
    let command_line_ptr = command_line_cstr.as_ptr() as *mut u8;

    // Wrap it in PSTR
    let command_line = PSTR(command_line_ptr);

    // Step 5: Initialize PROCESS_INFORMATION
    let mut process_info = PROCESS_INFORMATION::default();

    // Step 6: Create the PowerShell process
    let success = unsafe {
        CreateProcessA(
            None,              // lpApplicationName
            command_line,      // lpCommandLine
            None,              // lpProcessAttributes
            None,              // lpThreadAttributes
            TRUE,              // bInheritHandles
            CREATE_NO_WINDOW,  // dwCreationFlags
            None,              // lpEnvironment
            None,              // lpCurrentDirectory
            &mut startup_info, // lpStartupInfo
            &mut process_info, // lpProcessInformation
        )
    };

    if !success.is_ok() {
        error!("Failed to create PowerShell process.");
        return Err(anyhow::anyhow!("Failed to execute PowerShell script"));
    }

    info!("PowerShell script started successfully.");

    // Close the write ends of the pipes in the parent process
    unsafe {
        let _ = CloseHandle(stdout_write);
        let _ = CloseHandle(stderr_write);
    }

    // Step 7: Read from stdout and stderr
    let stdout =
        read_from_pipe(stdout_read).context("Failed to read stdout from PowerShell script")?;
    let stderr =
        read_from_pipe(stderr_read).context("Failed to read stderr from PowerShell script")?;

    // Step 8: Wait for the process to complete
    let wait_result = unsafe { WaitForSingleObject(process_info.hProcess, INFINITE) };

    if wait_result != WAIT_OBJECT_0 {
        error!(
            "WaitForSingleObject failed with return value: {:?}",
            wait_result
        );
        // Clean up handles before returning
        unsafe {
            let _ = CloseHandle(process_info.hProcess);
            let _ = CloseHandle(process_info.hThread);
            let _ = CloseHandle(stdout_read);
            let _ = CloseHandle(stderr_read);
        }
        return Err(anyhow::anyhow!("Failed to wait for PowerShell process"));
    }

    info!("PowerShell process has terminated successfully.");

    // Step 9: Close process and thread handles
    unsafe {
        let _ = CloseHandle(process_info.hProcess);
        let _ = CloseHandle(process_info.hThread);
        let _ = CloseHandle(stdout_read);
        let _ = CloseHandle(stderr_read);
    }

    // Step 10: Check if there was any error output
    if !stderr.is_empty() {
        error!("PowerShell script error output: {}", stderr.trim());
    }

    // You can choose to handle stderr separately or include it in the output
    // Here, we'll return only stdout for successful execution
    if !stderr.is_empty() {
        return Err(anyhow::anyhow!(
            "PowerShell script error: {}",
            stderr.trim()
        ));
    }

    Ok(stdout.trim().to_string())
}

// Helper function to read from a pipe handle until EOF
fn read_from_pipe(handle: HANDLE) -> Result<String> {
    let mut buffer = [0u8; 4096];
    let mut output = Vec::new();
    let mut bytes_read: u32 = 0;

    loop {
        let success =
            unsafe { ReadFile(handle, Some(&mut buffer), Some(&mut bytes_read), None).is_ok() };

        if !success {
            // You might want to handle specific errors here
            break;
        }

        if bytes_read == 0 {
            // EOF reached
            break;
        }

        output.extend_from_slice(&buffer[..bytes_read as usize]);
    }

    let output_str = String::from_utf8_lossy(&output).to_string();
    Ok(output_str)
}
