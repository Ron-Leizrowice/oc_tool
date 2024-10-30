use std::{thread, time::Duration};

use windows::core::{Error, Result, HRESULT};

use crate::utils::winring0::WINRING0_DRIVER;

// Constants
const NB_PCI_REG_ADDR_ADDR: u32 = 0xB8;
const NB_PCI_REG_DATA_ADDR: u32 = 0xBC;

// PCI configuration
const AMD_VENDOR_ID: u16 = 0x1022;
const PCI_CONFIG_SPACE: u32 = 0x00;

// Known SMU base addresses to try
const SMU_BASE_ADDRESSES: [u32; 4] = [
    0x3B10000,  // Original Ryzen
    0x03B10000, // Alternative notation
    0x0B10000,  // Some newer chipsets
    0x00B10000, // Another variation
];

// Message IDs
const SMU_TEST_MSG: u32 = 0x1;
const SMU_GET_VERSION: u32 = 0x2;

// Relative offsets (these seem consistent across chipsets)
const MP1_RELATIVE_OFFSETS: (u32, u32, u32) = (0x528, 0x564, 0x998);
const PSMU_RELATIVE_OFFSETS: (u32, u32, u32) = (0xa20, 0xa80, 0xa88);

const E_FAIL: i32 = 0x80004005u32 as i32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmuType {
    MP1,
    PSMU,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum SmuResponse {
    Ok = 0x1,
    Failed = 0xFF,
    UnknownCmd = 0xFE,
    CmdRejectedPrereq = 0xFD,
    CmdRejectedBusy = 0xFC,
}

impl SmuResponse {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x1 => Some(Self::Ok),
            0xFF => Some(Self::Failed),
            0xFE => Some(Self::UnknownCmd),
            0xFD => Some(Self::CmdRejectedPrereq),
            0xFC => Some(Self::CmdRejectedBusy),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct SmuArgs {
    pub arg0: u32,
    pub arg1: u32,
    pub arg2: u32,
    pub arg3: u32,
    pub arg4: u32,
    pub arg5: u32,
}

impl SmuArgs {
    fn as_array(&self) -> [u32; 6] {
        [
            self.arg0, self.arg1, self.arg2, self.arg3, self.arg4, self.arg5,
        ]
    }
}

pub struct SmuInterface {
    msg_addr: u32,
    rep_addr: u32,
    arg_base: u32,
}

impl SmuInterface {
    pub fn new(smu_type: SmuType) -> Result<Self> {
        // Verify AMD chip
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|e| Error::new(HRESULT(E_FAIL), format!("Failed to lock WinRing0: {}", e)))?;

        let pci_config = winring0.read_pci_config(PCI_CONFIG_SPACE)?;
        let vendor_id = pci_config & 0xFFFF;
        let device_id = pci_config >> 16;

        tracing::info!("PCI Config Space: {:#x}", pci_config);
        tracing::info!("Vendor ID: {:#x}", vendor_id);
        tracing::info!("Device ID: {:#x}", device_id);

        if vendor_id != AMD_VENDOR_ID as u32 {
            return Err(Error::new(
                HRESULT(E_FAIL),
                format!("Not an AMD chip. Vendor ID: {:#x}", vendor_id),
            ));
        }

        let (msg_offset, rep_offset, arg_base_offset) = match smu_type {
            SmuType::MP1 => {
                tracing::info!("Initializing MP1 SMU");
                MP1_RELATIVE_OFFSETS
            }
            SmuType::PSMU => {
                tracing::info!("Initializing PSMU SMU");
                PSMU_RELATIVE_OFFSETS
            }
        };

        // Try each known base address
        for &base_addr in &SMU_BASE_ADDRESSES {
            tracing::info!("Trying SMU base address: {:#x}", base_addr);

            let smu = Self {
                msg_addr: base_addr + msg_offset,
                rep_addr: base_addr + rep_offset,
                arg_base: base_addr + arg_base_offset,
            };

            tracing::info!("Testing addresses:");
            tracing::info!("  Message: {:#x}", smu.msg_addr);
            tracing::info!("  Response: {:#x}", smu.rep_addr);
            tracing::info!("  Arg base: {:#x}", smu.arg_base);

            // Try to read the response register
            match smu.smn_reg_read(smu.rep_addr) {
                Ok(value) => {
                    tracing::info!("Successfully read response register: {:#x}", value);

                    // Try sending a test message
                    let mut args = SmuArgs::default();
                    match smu.send_message(SMU_TEST_MSG, &mut args) {
                        Ok(SmuResponse::Ok) => {
                            tracing::info!("Successfully initialized SMU at base {:#x}", base_addr);
                            return Ok(smu);
                        }
                        Ok(response) => {
                            tracing::warn!("SMU test message returned: {:?}", response);
                        }
                        Err(e) => {
                            tracing::warn!("SMU test message failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to read response register at base {:#x}: {:?}",
                        base_addr,
                        e
                    );
                }
            }

            // Small delay before trying next base address
            thread::sleep(Duration::from_millis(100));
        }

        Err(Error::new(
            HRESULT(E_FAIL),
            format!(
                "Could not find valid SMU base address for device ID: {:#x}",
                device_id
            ),
        ))
    }

    fn smn_reg_read(&self, addr: u32) -> Result<u32> {
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|e| Error::new(HRESULT(E_FAIL), format!("Failed to lock WinRing0: {}", e)))?;

        // Write address first with different masking attempts
        for &addr_mask in &[addr, addr & !0x3, addr | 0x3] {
            winring0.write_pci_config(NB_PCI_REG_ADDR_ADDR, addr_mask)?;
            thread::sleep(Duration::from_micros(10));

            let value = winring0.read_pci_config(NB_PCI_REG_DATA_ADDR)?;
            if value != 0xFFFFFFFF {
                // Valid data typically isn't all 1s
                tracing::debug!(
                    "SMN READ: Addr [{:#x}] = {:#x} (mask: {:#x})",
                    addr,
                    value,
                    addr_mask
                );
                return Ok(value);
            }
        }

        tracing::debug!("SMN READ FAILED: Addr [{:#x}]", addr);
        Ok(0) // Return 0 instead of error for invalid reads
    }

    fn smn_reg_write(&self, addr: u32, data: u32) -> Result<()> {
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|e| Error::new(HRESULT(E_FAIL), format!("Failed to lock WinRing0: {}", e)))?;

        // Try different address masks
        for &addr_mask in &[addr, addr & !0x3, addr | 0x3] {
            winring0.write_pci_config(NB_PCI_REG_ADDR_ADDR, addr_mask)?;
            thread::sleep(Duration::from_micros(10));
            winring0.write_pci_config(NB_PCI_REG_DATA_ADDR, data)?;

            // Verify write
            thread::sleep(Duration::from_micros(10));
            let verify = winring0.read_pci_config(NB_PCI_REG_DATA_ADDR)?;
            if verify == data {
                tracing::debug!(
                    "SMN WRITE: Addr [{:#x}] = {:#x} (mask: {:#x})",
                    addr,
                    data,
                    addr_mask
                );
                return Ok(());
            }
        }

        tracing::debug!("SMN WRITE: Addr [{:#x}] = {:#x} (unverified)", addr, data);
        Ok(())
    }

    pub fn send_message(&self, msg: u32, args: &mut SmuArgs) -> Result<SmuResponse> {
        tracing::info!(
            "Sending SMU message {:#x} with args: {:x?}",
            msg,
            args.as_array()
        );

        // Clear response register
        self.smn_reg_write(self.rep_addr, 0)?;
        thread::sleep(Duration::from_micros(100));

        // Write arguments
        for (i, &arg) in args.as_array().iter().enumerate() {
            self.smn_reg_write(self.arg_base + (i as u32 * 4), arg)?;
            thread::sleep(Duration::from_micros(10));
        }

        // Send message
        self.smn_reg_write(self.msg_addr, msg)?;
        thread::sleep(Duration::from_micros(100));

        // Wait for response with a busy loop
        let mut response = 0;
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 1000;

        while response == 0 && attempts < MAX_ATTEMPTS {
            response = self.smn_reg_read(self.rep_addr)?;
            if response != 0 && response != 0xFFFFFFFF {
                tracing::debug!("Got response after {} attempts: {:#x}", attempts, response);
                break;
            }
            attempts += 1;
            thread::sleep(Duration::from_micros(100));
        }

        if response == 0 || response == 0xFFFFFFFF {
            tracing::error!("No valid response after {} attempts", MAX_ATTEMPTS);
            return Err(Error::new(HRESULT(E_FAIL), "No valid response from SMU"));
        }

        // Read back arguments
        for i in 0..6 {
            args.as_array()[i] = self.smn_reg_read(self.arg_base + (i as u32 * 4))?;
        }

        let smu_response = SmuResponse::from_u32(response).ok_or_else(|| {
            let msg = format!("Invalid SMU response: {:#x}", response);
            tracing::error!("{}", msg);
            Error::new(HRESULT(E_FAIL), msg)
        })?;

        tracing::info!(
            "SMU response: {:?}, returned args: {:x?}",
            smu_response,
            args.as_array()
        );

        Ok(smu_response)
    }

    pub fn get_version(&self) -> Result<(u32, u32)> {
        let mut args = SmuArgs::default();
        match self.send_message(SMU_GET_VERSION, &mut args)? {
            SmuResponse::Ok => Ok((args.arg0, args.arg1)),
            response => {
                let msg = format!("Failed to get SMU version: {:?}", response);
                tracing::error!("{}", msg);
                Err(Error::new(HRESULT(E_FAIL), msg))
            }
        }
    }
}
