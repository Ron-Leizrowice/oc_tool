// src/utils/winring0.rs

use std::{ffi::CStr, sync::Mutex};

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
    utils::registry::{create_or_modify_registry_value, RegistryKeyValue},
};

// Define constants
const WINRING0_DLL: &str = "WinRing0x64.dll";

static DISABLE_HYPERVISOR_ENFORCED_CODE_INTEGRITY: Lazy<Vec<RegistryModification>> = Lazy::new(
    || {
        vec![
            RegistryModification {
                path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
                key: "WasEnabledBy",
                enabled: RegistryKeyValue::Dword(1),
                disabled: None,
            },
            RegistryModification {
                path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
                key: "Enabled",
                enabled: RegistryKeyValue::Dword(0),
                disabled: Some(RegistryKeyValue::Dword(1)),
            },
            RegistryModification {
                path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
                key: "EnabledBootId",
                enabled: RegistryKeyValue::Dword(0),
                disabled: Some(RegistryKeyValue::Dword(1)),
            },
        ]
    },
);

static DISABLE_VULNERABLE_DRIVER_BLOCKLIST: Lazy<Vec<RegistryModification>> = Lazy::new(|| {
    vec![RegistryModification {
        path: r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\CI\Config",
        key: "VulnerableDriverBlocklistEnable",
        enabled: RegistryKeyValue::Dword(0),
        disabled: Some(RegistryKeyValue::Dword(1)),
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
        let result = unsafe { (self.read_msr)(index, &mut eax, &mut edx, affinity_mask) };

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

// Modify the setup_winring0_service function
pub fn setup_winring0_driver() -> anyhow::Result<()> {
    // 1. Check that the WinRing0 DLL is in the current directory
    let winring0_dll_path = std::env::current_dir().unwrap().join(WINRING0_DLL);
    if !winring0_dll_path.exists() {
        return Err(anyhow::anyhow!(
            "WinRing0 DLL not found in current directory: {:?}",
            winring0_dll_path
        ));
    }

    // Create/modify the registry entries for disabling the Vulnerable Driver Blocklist
    for modification in DISABLE_VULNERABLE_DRIVER_BLOCKLIST.iter() {
        if let Err(e) = create_or_modify_registry_value(
            modification.path,
            modification.key,
            &modification.enabled,
        ) {
            return Err(anyhow::anyhow!(
                "Failed to create or modify registry value: {:?}",
                e
            ));
        }
    }

    // Create/modify the registry entries for disabling Hypervisor Enforced Code Integrity
    for modification in DISABLE_HYPERVISOR_ENFORCED_CODE_INTEGRITY.iter() {
        if let Err(e) = create_or_modify_registry_value(
            modification.path,
            modification.key,
            &modification.enabled,
        ) {
            return Err(anyhow::anyhow!(
                "Failed to create or modify registry value: {:?}",
                e
            ));
        }
    }

    Ok(())
}
