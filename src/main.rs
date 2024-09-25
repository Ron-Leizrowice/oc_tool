// src/main.rs
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr::null_mut};

use druid::{AppLauncher, LocalizedString, WindowDesc};
use oc_tool::{models::AppState, ui::build_root_widget};
use winapi::um::{
    handleapi::CloseHandle,
    processthreadsapi::{GetCurrentProcess, OpenProcessToken},
    securitybaseapi::GetTokenInformation,
    winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY},
    winuser::{MessageBoxW, MB_ICONWARNING, MB_OK},
};

fn is_elevated() -> bool {
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
            return elevation.TokenIsElevated != 0;
        }
    }
    if !handle.is_null() {
        unsafe { CloseHandle(handle) };
    }
    false
}

#[tokio::main]
async fn main() {
    if !is_elevated() {
        // Display a message box
        let caption = "cOCaine";
        let message = "cOCaine must be run as administrator.";

        let caption_wide: Vec<u16> = OsStr::new(caption)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let message_wide: Vec<u16> = OsStr::new(message)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            MessageBoxW(
                null_mut(),             // No owner window
                message_wide.as_ptr(),  // Message text
                caption_wide.as_ptr(),  // Caption text
                MB_OK | MB_ICONWARNING, // OK button + Warning icon
            );
        }
        return;
    }
    let main_window = WindowDesc::new(build_root_widget())
        .title(LocalizedString::new("cOCaine"))
        .window_size((500.0, 400.0));

    let initial_state = AppState::default();

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("launch failed");
}
