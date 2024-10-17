// src/tweaks/msr.rs

use windows::{
    core::*,
    Win32::{
        Foundation::{GetLastError, BOOL, E_FAIL, HMODULE},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
};

use super::{definitions::TweakId, powershell::execute_powershell_script, TweakMethod};

pub const WINRING0_DLL: &str = "WinRing0x64.dll";
pub const WINRING0_SYS: &str = "WinRing0x64.sys";

// Ensure MSRTweak implements Clone
#[derive(Clone)]
pub struct MSRTweak {
    pub id: TweakId,
    pub index: u32,
    pub bit: u32,
}

impl TweakMethod for MSRTweak {
    fn initial_state(&self) -> std::result::Result<bool, anyhow::Error> {
        let winring0 = WinRing0::new()?;
        let mut all_states = Vec::new();

        for core_id in 0..num_cpus::get() {
            let value = winring0.read_msr(core_id, self.index)?;
            let state = (value >> self.bit) & 1 == 1;
            all_states.push(state);
        }

        // Verify all cores have the same state
        if all_states.iter().all(|&state| state == all_states[0]) {
            Ok(all_states[0])
        } else {
            Ok(false) // Return false if states are not consistent
        }
    }

    // Similarly update `apply` and `revert` methods
    fn apply(&self) -> std::result::Result<(), anyhow::Error> {
        let winring0 = WinRing0::new()?;
        for core_id in 0..num_cpus::get() {
            let current_value = winring0.read_msr(core_id, self.index)?;
            let new_value = current_value | (1 << self.bit); // Set bit
            winring0.write_msr(core_id, self.index, new_value)?;

            // Verify the bit is set
            let updated_value = winring0.read_msr(core_id, self.index)?;
            let state = (updated_value >> self.bit) & 1 == 1;
            match state {
                true => {}
                false => {
                    tracing::error!("Failed to set MSR bit {} on core {}", self.bit, core_id);
                    return Err(anyhow::anyhow!(
                        "Failed to set MSR bit {} on core {}",
                        self.bit,
                        core_id
                    ));
                }
            }
        }

        tracing::info!(
            "Successfully applied MSR 0x{:X} bit {} on all cores.",
            self.index,
            self.bit,
        );

        Ok(())
    }

    fn revert(&self) -> std::result::Result<(), anyhow::Error> {
        let winring0 = WinRing0::new()?;
        for core_id in 0..num_cpus::get() {
            let current_value = winring0.read_msr(core_id, self.index)?;
            let new_value = current_value & !(1 << self.bit); // Clear bit
            winring0.write_msr(core_id, self.index, new_value)?;

            // Verify the bit is cleared
            let updated_value = winring0.read_msr(core_id, self.index)?;
            let state = (updated_value >> self.bit) & 1 == 0;
            match state {
                true => {}
                false => {
                    tracing::error!("Failed to clear MSR bit {} on core {}", self.bit, core_id);
                    return Err(anyhow::anyhow!(
                        "Failed to clear MSR bit {} on core {}",
                        self.bit,
                        core_id
                    ));
                }
            }
        }

        tracing::info!(
            "Successfully reverted MSR 0x{:X} bit {} on all cores.",
            self.index,
            self.bit,
        );

        Ok(())
    }
}

type InitializeOls = unsafe extern "system" fn() -> BOOL;
type DeinitializeOls = unsafe extern "system" fn();
type ReadMsrTx = unsafe extern "system" fn(u32, *mut u32, *mut u32, u32) -> BOOL;
type WriteMsrTx = unsafe extern "system" fn(u32, u32, u32, u32) -> BOOL;

pub struct WinRing0 {
    _lib: HMODULE,
    _initialize: InitializeOls,
    deinitialize: DeinitializeOls,
    read_msr: ReadMsrTx,
    write_msr: WriteMsrTx,
}

impl WinRing0 {
    pub fn new() -> windows::core::Result<Self> {
        let winring0_dll_path = std::env::current_dir()?.join("WinRing0x64.dll");

        unsafe {
            let dll_path_w: Vec<u16> = winring0_dll_path
                .to_str()
                .ok_or_else(|| Error::new(E_FAIL, "Failed to convert PathBuf to str"))?
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let lib = LoadLibraryW(PCWSTR(dll_path_w.as_ptr()))?;

            let _initialize: InitializeOls = std::mem::transmute(
                GetProcAddress(lib, PCSTR(b"InitializeOls\0".as_ptr()))
                    .ok_or_else(|| Error::new(E_FAIL, "Failed to get InitializeOls"))?,
            );

            let deinitialize: DeinitializeOls = std::mem::transmute(
                GetProcAddress(lib, PCSTR(b"DeinitializeOls\0".as_ptr()))
                    .ok_or(Error::new(E_FAIL, "Failed to get DeinitializeOls"))?,
            );
            let read_msr: ReadMsrTx = std::mem::transmute(
                GetProcAddress(lib, PCSTR(b"RdmsrTx\0".as_ptr()))
                    .ok_or(Error::new(E_FAIL, "Failed to get Rdmsr"))?,
            );
            let write_msr: WriteMsrTx = std::mem::transmute(
                GetProcAddress(lib, PCSTR(b"WrmsrTx\0".as_ptr()))
                    .ok_or(Error::new(E_FAIL, "Failed to get Wrmsr"))?,
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
        let affinity_mask = 1 << core_id;
        let result = unsafe {
            let x = (self.read_msr)(index, &mut eax, &mut edx, affinity_mask);
            tracing::debug!("Read MSR: 0x{:X} on Core {} -> {:?}", index, core_id, x);
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
        let affinity_mask = 1 << core_id;
        let result = unsafe {
            tracing::debug!(
                "Write MSR: 0x{:X} <- 0x{:X}{:X} on Core {}",
                index,
                edx,
                eax,
                core_id
            );
            (self.write_msr)(index, eax, edx, affinity_mask)
        };

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

pub fn verify_winring0_setup() -> anyhow::Result<()> {
    // check that the winring dll is in system32
    let winring0_dll_path = std::env::current_dir()?.join(WINRING0_DLL);
    if !winring0_dll_path.exists() {
        return Err(anyhow::anyhow!("WinRing0 DLL not found"));
    }

    // check that the winring service is running
    let powershell_cmd =
        r#"(Get-Service -Name WinRing0x64 | Select-Object -Property Status) -Match "Running""#;

    let output = std::process::Command::new("powershell")
        .arg("-Command")
        .arg(powershell_cmd)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run PowerShell command: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "WinRing0 service is not running. Please start the service and try again."
        ));
    } else if !String::from_utf8_lossy(&output.stdout).contains("True") {
        return Err(anyhow::anyhow!(
            "WinRing0 service is not running. Please start the service and try again."
        ));
    }

    Ok(())
}

pub fn setup_winring0_service() -> anyhow::Result<()> {
    // check that the winring dll is in system32
    let winring0_dll_path = std::env::current_dir()?.join(WINRING0_DLL);
    if !winring0_dll_path.exists() {
        return Err(anyhow::anyhow!("WinRing0 DLL not found"));
    }

    let winring0_sys_path = std::env::current_dir()?.join(WINRING0_SYS);
    if !winring0_sys_path.exists() {
        return Err(anyhow::anyhow!("WinRing0.sys not found"));
    }

    // create and start the service
    let script = r#"
    Stop-Service -Name WinRing0x64 -ErrorAction Continue
    sc.exe delete WinRing0x64 -ErrorAction Continue
    sc.exe create WinRing0x64 type= kernel binPath= $sysDestination start=auto
    Start-Service -Name WinRing0x64 -ErrorAction Stop
"#;
    match execute_powershell_script(script) {
        Ok(_) => Ok(()),
        Err(e) => return Err(anyhow::anyhow!("Failed to create WinRing0 service: {}", e)),
    }
}
