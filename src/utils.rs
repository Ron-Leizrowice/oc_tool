// src/utils.rs

use windows::Win32::{
    Foundation::NTSTATUS,
    Security::Authentication::Identity::{LsaClose, LsaNtStatusToWinError, LSA_HANDLE},
};

pub struct LsaHandleGuard {
    pub handle: LSA_HANDLE,
}

impl Drop for LsaHandleGuard {
    fn drop(&mut self) {
        unsafe {
            let status = LsaClose(self.handle);
            if status != NTSTATUS(0) {
                eprintln!(
                    "LsaClose failed with error code: {}",
                    LsaNtStatusToWinError(status)
                );
            }
        }
    }
}
