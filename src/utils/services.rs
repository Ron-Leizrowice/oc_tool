// src/utils/services.rs

use windows::{
    core::PCWSTR,
    Win32::System::Services::{
        CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus, StartServiceW,
        SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START, SERVICE_STATUS,
    },
};

/// Checks if a specified Windows service is currently running.
///
/// # Parameters
///
/// - `service_name`: The name of the service to check.
///
/// # Returns
///
/// - `Ok(true)` if the service is running.
/// - `Ok(false)` if the service is not running.
/// - `Err(anyhow::Error)` if an error occurs while querying the service.
pub fn is_service_running(service_name: &str) -> anyhow::Result<bool> {
    unsafe {
        // Open the Service Control Manager
        let scm_handle = match OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT) {
            Ok(handle) => handle,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "Failed to open Service Control Manager: {:?}",
                    windows::core::Error::from_win32()
                ));
            }
        };

        // Open the specified service
        let service_handle = match OpenServiceW(
            scm_handle,
            PCWSTR::from_raw(widestring::U16CString::from_str(service_name)?.as_ptr()),
            SERVICE_QUERY_STATUS,
        ) {
            Ok(handle) => handle,
            Err(_) => {
                match CloseServiceHandle(scm_handle) {
                    Ok(_) => (),
                    Err(_) => {
                        panic!(
                            "Failed to close Service Control Manager handle: {:?}",
                            windows::core::Error::from_win32()
                        );
                    }
                }
                return Err(anyhow::anyhow!(
                    "Failed to open service '{}': {:?}",
                    service_name,
                    windows::core::Error::from_win32()
                ));
            }
        };

        // Query the service status
        let mut status = SERVICE_STATUS::default();
        let query_result = QueryServiceStatus(service_handle, &mut status);

        // Close handles
        match CloseServiceHandle(service_handle) {
            Ok(_) => (),
            Err(_) => {
                panic!(
                    "Failed to close service handle: {:?}",
                    windows::core::Error::from_win32()
                );
            }
        }
        match CloseServiceHandle(scm_handle) {
            Ok(_) => (),
            Err(_) => {
                panic!(
                    "Failed to close Service Control Manager handle: {:?}",
                    windows::core::Error::from_win32()
                );
            }
        }

        if query_result.is_ok() {
            Ok(status.dwCurrentState == SERVICE_RUNNING)
        } else {
            Err(anyhow::anyhow!(
                "Failed to query service status for '{}': {:?}",
                service_name,
                windows::core::Error::from_win32()
            ))
        }
    }
}

/// Starts a specified Windows service.
///
/// # Parameters
/// - `service_name`: The name of the service to start.
///
/// # Returns
/// - `Ok(())` if the service was successfully started.
/// - `Err(anyhow::Error)` if an error occurs while starting the service.
pub fn start_service(service_name: &str) -> anyhow::Result<()> {
    unsafe {
        // Open the Service Control Manager
        let scm_handle = match OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT) {
            Ok(handle) => handle,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "Failed to open Service Control Manager: {:?}",
                    windows::core::Error::from_win32()
                ));
            }
        };

        // Open the specified service with SERVICE_START and SERVICE_QUERY_STATUS permissions
        let service_handle = match OpenServiceW(
            scm_handle,
            PCWSTR::from_raw(widestring::U16CString::from_str(service_name)?.as_ptr()),
            SERVICE_START | SERVICE_QUERY_STATUS, // Updated permissions
        ) {
            Ok(handle) => handle,
            Err(_) => {
                match CloseServiceHandle(scm_handle) {
                    Ok(_) => (),
                    Err(_) => {
                        panic!(
                            "Failed to close Service Control Manager handle: {:?}",
                            windows::core::Error::from_win32()
                        );
                    }
                }
                return Err(anyhow::anyhow!(
                    "Failed to open service '{}': {:?}",
                    service_name,
                    windows::core::Error::from_win32()
                ));
            }
        };

        // Start the service
        let start_result = StartServiceW(
            service_handle, // hservice
            None,           // lpserviceargvectors
        );

        // Close handles
        match CloseServiceHandle(service_handle) {
            Ok(_) => (),
            Err(_) => {
                panic!(
                    "Failed to close service handle: {:?}",
                    windows::core::Error::from_win32()
                );
            }
        }
        match CloseServiceHandle(scm_handle) {
            Ok(_) => (),
            Err(_) => {
                panic!(
                    "Failed to close Service Control Manager handle: {:?}",
                    windows::core::Error::from_win32()
                );
            }
        }

        if start_result.is_ok() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to start service '{}': {:?}",
                service_name,
                windows::core::Error::from_win32()
            ))
        }
    }
}
