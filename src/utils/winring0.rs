// src/utils/winring0.rs

use std::{ffi::CStr, sync::Mutex};

use anyhow::Context;
use cstr::cstr;
use once_cell::sync::Lazy;
use windows::{
    core::*,
    Win32::{
        Foundation::{GetLastError, BOOL, E_FAIL, HMODULE},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
};

use crate::{
    tweaks::registry::method::RegistryModification,
    utils::{
        registry::{create_or_modify_registry_value, read_registry_value, RegistryKeyValue},
        services::{is_service_running, start_service},
    },
};

// Define constants
const SERVICE_NAME: &str = "WinRing0x64";
const WINRING0_DLL: &str = "WinRing0x64.dll";
const WINRING0_SYS: &str = "WinRing0x64.sys";

static WINRING0_SERVICE: Lazy<Vec<RegistryModification>> = Lazy::new(|| {
    vec![
        RegistryModification {
            path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\WinRing0x64",
            key: "Type",
            target_value: RegistryKeyValue::Dword(1),
            default_value: None,
        },
        RegistryModification {
            path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\WinRing0x64",
            key: "Start",
            target_value: RegistryKeyValue::Dword(1),
            default_value: None,
        },
        RegistryModification {
            path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\WinRing0x64",
            key: "ErrorControl",
            target_value: RegistryKeyValue::Dword(1),
            default_value: None,
        },
        RegistryModification {
            path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\WinRing0x64",
            key: "ImagePath",
            target_value: RegistryKeyValue::String(
                std::env::current_dir()
                    .unwrap()
                    .join(WINRING0_SYS)
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            default_value: None,
        },
    ]
});

static DISABLE_HYPERVISOR_ENFORCED_CODE_INTEGRITY: Lazy<Vec<RegistryModification>> = Lazy::new(
    || {
        vec![
            RegistryModification {
                path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
                key: "WasEnabledBy",
                target_value: RegistryKeyValue::Dword(1),
                default_value: None,
            },
            RegistryModification {
                path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
                key: "Enabled",
                target_value: RegistryKeyValue::Dword(0),
                default_value: Some(RegistryKeyValue::Dword(1)),
            },
            RegistryModification {
                path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
                key: "EnabledBootId",
                target_value: RegistryKeyValue::Dword(0),
                default_value: Some(RegistryKeyValue::Dword(1)),
            },
        ]
    },
);

static DISABLE_VULNERABLE_DRIVER_BLOCKLIST: Lazy<Vec<RegistryModification>> = Lazy::new(|| {
    vec![RegistryModification {
        path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\CI\Config",
        key: "VulnerableDriverBlocklistEnable",
        target_value: RegistryKeyValue::Dword(0),
        default_value: Some(RegistryKeyValue::Dword(1)),
    }]
});

static INITIALIZE_OLS: &CStr = cstr!("InitializeOls");
static DEINITIALIZE_OLS: &CStr = cstr!("DeinitializeOls");
static WRMSRTX: &CStr = cstr!("WrmsrTx");
static RDMSRTX: &CStr = cstr!("RdmsrTx");

type InitializeOls = unsafe extern "system" fn() -> BOOL;
type DeinitializeOls = unsafe extern "system" fn();
type ReadMsrTx = unsafe extern "system" fn(u32, *mut u32, *mut u32, u64) -> BOOL;
type WriteMsrTx = unsafe extern "system" fn(u32, u32, u32, u64) -> BOOL;

pub struct WinRing0 {
    _lib: HMODULE,
    _initialize: InitializeOls,
    deinitialize: DeinitializeOls,
    read_msr: ReadMsrTx,
    write_msr: WriteMsrTx,
}

// Ensure WinRing0 is Send and Sync
unsafe impl Send for WinRing0 {}
unsafe impl Sync for WinRing0 {}

// Initialize the singleton instance
pub static WINRING0_DRIVER: Lazy<Mutex<WinRing0>> =
    Lazy::new(|| Mutex::new(WinRing0::new().expect("Failed to initialize WinRing0")));

impl WinRing0 {
    pub fn new() -> windows::core::Result<Self> {
        let winring0_dll_path = std::env::current_dir()?.join(WINRING0_DLL);

        unsafe {
            let dll_path_w: Vec<u16> = winring0_dll_path
                .to_str()
                .ok_or_else(|| Error::new(E_FAIL, "Failed to convert PathBuf to str"))?
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let lib = LoadLibraryW(PCWSTR(dll_path_w.as_ptr()))?;

            let _initialize: InitializeOls = std::mem::transmute(
                GetProcAddress(lib, PCSTR(INITIALIZE_OLS.as_ptr() as *const u8))
                    .ok_or_else(|| Error::new(E_FAIL, "Failed to get InitializeOls"))?,
            );

            let deinitialize: DeinitializeOls = std::mem::transmute(
                GetProcAddress(lib, PCSTR(DEINITIALIZE_OLS.as_ptr() as *const u8))
                    .ok_or(Error::new(E_FAIL, "Failed to get DeinitializeOls"))?,
            );
            let read_msr: ReadMsrTx = std::mem::transmute(
                GetProcAddress(lib, PCSTR(RDMSRTX.as_ptr() as *const u8))
                    .ok_or(Error::new(E_FAIL, "Failed to get RdmsrTx"))?,
            );
            let write_msr: WriteMsrTx = std::mem::transmute(
                GetProcAddress(lib, PCSTR(WRMSRTX.as_ptr() as *const u8))
                    .ok_or(Error::new(E_FAIL, "Failed to get WrmsrTx"))?,
            );

            // **Initialize WinRing0**
            let init_result = _initialize();
            if !init_result.as_bool() {
                let error_code = GetLastError();
                return Err(Error::new(
                    E_FAIL,
                    format!(
                        "Failed to initialize WinRing0. GetLastError: {}",
                        error_code.0
                    ),
                ));
            }

            Ok(WinRing0 {
                _lib: lib,
                _initialize,
                deinitialize,
                read_msr,
                write_msr,
            })
        }
    }

    /// Reads the MSR for a specific core.
    pub fn read_msr(&self, core_id: usize, index: u32) -> windows::core::Result<u64> {
        // Perform the MSR read operation
        let mut eax: u32 = 0;
        let mut edx: u32 = 0;
        let affinity_mask = 1u64 << core_id;
        let result = unsafe {
            let x = (self.read_msr)(index, &mut eax, &mut edx, affinity_mask);

            x
        };

        match result.as_bool() {
            true => Ok(((edx as u64) << 32) | eax as u64),
            false => Err(Error::new(
                E_FAIL,
                format!("Failed to read MSR 0x{:X} on core {}", index, core_id),
            )),
        }
    }

    /// Writes to the MSR for a specific core.
    pub fn write_msr(&self, core_id: usize, index: u32, value: u64) -> windows::core::Result<()> {
        // Perform the MSR write operation
        let eax = (value & 0xFFFFFFFF) as u32;
        let edx = (value >> 32) as u32;
        let affinity_mask = 1u64 << core_id;
        let result = unsafe { (self.write_msr)(index, eax, edx, affinity_mask) };

        match result.as_bool() {
            true => Ok(()),
            false => Err(Error::new(
                E_FAIL,
                format!("Failed to write MSR 0x{:X} on core {}", index, core_id),
            )),
        }
    }
}

impl Drop for WinRing0 {
    fn drop(&mut self) {
        unsafe {
            (self.deinitialize)();
        }
    }
}

/// Verifies that the WinRing0x64 driver and related registry configurations are properly set up.
pub fn verify_winring0_setup() -> anyhow::Result<()> {
    // 1. Check that the WinRing0 DLL is in the current directory
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let winring0_dll_path = current_dir.join(WINRING0_DLL);
    if !winring0_dll_path.exists() {
        return Err(anyhow::anyhow!(
            "WinRing0 DLL not found at {:?}",
            winring0_dll_path
        ));
    }

    // 2. Check that the WinRing0x64 service is running
    match is_service_running(SERVICE_NAME) {
        Ok(true) => {
            tracing::debug!("WinRing0x64 service is running.");
        }
        Ok(false) => {
            return Err(anyhow::anyhow!(
                "WinRing0x64 service is not running. Please ensure the service is started."
            ));
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to check if WinRing0x64 service is running: {}",
                e
            ));
        }
    }

    // 3. Verify the 'VulnerableDriverBlocklistEnable' registry settings
    for modification in DISABLE_VULNERABLE_DRIVER_BLOCKLIST.iter() {
        match read_registry_value(&modification.path, modification.key) {
            Ok(Some(value)) if value == modification.target_value => {
                tracing::debug!(
                    "Registry value '{}' in '{}' is set to {:?}",
                    modification.key,
                    modification.path,
                    value
                );
            }
            Ok(Some(value)) => {
                return Err(anyhow::anyhow!(
                    "Registry value '{}' in '{}' is set to {:?}. Expected {:?}",
                    modification.key,
                    modification.path,
                    value,
                    modification.target_value
                ));
            }
            Ok(None) => {
                return Err(anyhow::anyhow!(
                    "Registry value '{}' in '{}' not found. Expected {:?}",
                    modification.key,
                    modification.path,
                    modification.target_value
                ));
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to read registry value '{}' in '{}': {}",
                    modification.key,
                    modification.path,
                    e
                ));
            }
        }
    }

    // 4. Verify the 'HypervisorEnforcedCodeIntegrity' registry settings
    for modification in DISABLE_HYPERVISOR_ENFORCED_CODE_INTEGRITY.iter() {
        match read_registry_value(&modification.path, modification.key) {
            Ok(Some(value)) if value == modification.target_value => {
                tracing::debug!(
                    "Registry value '{}' in '{}' is set to {:?}",
                    modification.key,
                    modification.path,
                    value
                );
            }
            Ok(Some(value)) => {
                return Err(anyhow::anyhow!(
                    "Registry value '{}' in '{}' is set to {:?}. Expected {:?}",
                    modification.key,
                    modification.path,
                    value,
                    modification.target_value
                ));
            }
            Ok(None) => {
                return Err(anyhow::anyhow!(
                    "Registry value '{}' in '{}' not found. Expected {:?}",
                    modification.key,
                    modification.path,
                    modification.target_value
                ));
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to read registry value '{}' in '{}': {}",
                    modification.key,
                    modification.path,
                    e
                ));
            }
        }
    }

    tracing::debug!("WinRing0x64 driver correctly configured.");
    Ok(())
}

/// Sets up the WinRing0x64 service by creating/modifying the necessary registry entries.
/// This includes additional configurations for Code Integrity and Device Guard.
pub fn setup_winring0_service() -> anyhow::Result<()> {
    // Check that the WinRing0 DLL is in the current directory
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let winring0_dll_path = current_dir.join(WINRING0_DLL);
    if !winring0_dll_path.exists() {
        return Err(anyhow::anyhow!(
            "WinRing0 DLL not found at {:?}",
            winring0_dll_path
        ));
    }

    // Check that the WinRing0.sys is in the current directory
    let winring0_sys_path = current_dir.join(WINRING0_SYS);
    if !winring0_sys_path.exists() {
        return Err(anyhow::anyhow!(
            "WinRing0.sys not found at {:?}",
            winring0_sys_path
        ));
    }

    // Create/modify the registry entries for the WinRing0x64 service
    for modification in WINRING0_SERVICE.iter() {
        create_or_modify_registry_value(
            &modification.path,
            modification.key,
            &modification.target_value,
        )?;
    }

    // Create/modify the registry entries for disabling the Vulnerable Driver Blocklist
    for modification in DISABLE_VULNERABLE_DRIVER_BLOCKLIST.iter() {
        create_or_modify_registry_value(
            &modification.path,
            modification.key,
            &modification.target_value,
        )?;
    }

    // Create/modify the registry entries for disabling Hypervisor Enforced Code Integrity
    for modification in DISABLE_HYPERVISOR_ENFORCED_CODE_INTEGRITY.iter() {
        create_or_modify_registry_value(
            &modification.path,
            modification.key,
            &modification.target_value,
        )?;
    }

    // Start the WinRing0x64 service
    match start_service(SERVICE_NAME) {
        Ok(_) => {
            tracing::debug!("WinRing0x64 service started successfully.");
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to start WinRing0x64 service: {}",
                e
            ));
        }
    }

    // Check that the WinRing0x64 service is running
    match is_service_running(SERVICE_NAME) {
        Ok(true) => {
            tracing::debug!("WinRing0x64 service is running.");
        }
        Ok(false) => {
            return Err(anyhow::anyhow!(
                "WinRing0x64 service is not running. Please ensure the service is started."
            ));
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to check if WinRing0x64 service is running: {}",
                e
            ));
        }
    }

    Ok(())
}
