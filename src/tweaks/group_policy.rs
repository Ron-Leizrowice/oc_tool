// src/tweaks/group_policy_tweaks.rs

use std::{
    ptr,
    sync::{Arc, Mutex},
};

use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{GetLastError, NTSTATUS, STATUS_OBJECT_NAME_NOT_FOUND},
        Security::{
            Authentication::Identity::{
                LsaAddAccountRights, LsaClose, LsaEnumerateAccountRights, LsaFreeMemory,
                LsaNtStatusToWinError, LsaOpenPolicy, LsaRemoveAccountRights, LSA_HANDLE,
                LSA_OBJECT_ATTRIBUTES, LSA_UNICODE_STRING,
            },
            LookupAccountNameW, PSID, SID_NAME_USE,
        },
    },
};

use super::{method::TweakMethod, Tweak, TweakCategory, TweakId};

/// Group Policy related constants.
pub static POLICY_CREATE_ACCOUNT: u32 = 0x00000010;
pub static POLICY_LOOKUP_NAMES: u32 = 0x00000800;

/// Represents a Group Policy tweak, including the policy key and desired value.
#[derive(Clone, Debug)]
pub struct GroupPolicyTweak {
    /// The policy key (e.g., "SeLockMemoryPrivilege").
    pub key: String,
    /// The desired value for the policy.
    pub value: GroupPolicyValue,
}

/// Enumeration of possible Group Policy values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroupPolicyValue {
    Enabled,
    Disabled,
}

impl GroupPolicyTweak {
    /// Reads the current value of the Group Policy tweak.
    ///
    /// # Returns
    ///
    /// - `Ok(GroupPolicyValue)` indicating if the policy is enabled or disabled.
    /// - `Err(GroupPolicyError)` if the operation fails.
    pub fn read_current_value(&self, id: TweakId) -> Result<GroupPolicyValue, anyhow::Error> {
        tracing::info!("{:?} -> Reading current value of Group Policy tweak.", id,);

        unsafe {
            // Initialize object attributes for LsaOpenPolicy
            let object_attributes = LSA_OBJECT_ATTRIBUTES::default();
            // Initialize the policy handle to zero to avoid using uninitialized memory
            let mut policy_handle = LSA_HANDLE(0);
            // Define the desired access rights for the policy object handle (read-only)
            let desired_access = POLICY_LOOKUP_NAMES;
            // Call LsaOpenPolicy to get a handle to the policy object
            let status =
                LsaOpenPolicy(None, &object_attributes, desired_access, &mut policy_handle);
            // Check the return value of LsaOpenPolicy
            if status != NTSTATUS(0) {
                let win_err = LsaNtStatusToWinError(status);
                tracing::error!(
                    "{:?} -> LsaOpenPolicy failed with error code: {}",
                    id,
                    win_err
                );
                return Err(anyhow::Error::msg(format!(
                    "LsaOpenPolicy failed with error code: {}",
                    win_err
                )));
            }

            // Ensure the policy handle is closed properly
            let _policy_guard = LsaHandleGuard {
                handle: policy_handle,
            };

            // Get the SID for the current user to enumerate account rights
            let mut sid_size = 0u32;
            let mut domain_name_size = 0u32;
            let mut sid_name_use = SID_NAME_USE(0);

            let user_name = whoami::username();
            let user_name_wide: Vec<u16> = user_name.encode_utf16().chain(Some(0)).collect();

            // First call to LookupAccountNameW to get buffer sizes
            let lookup_result = LookupAccountNameW(
                PCWSTR(ptr::null()),
                PCWSTR(user_name_wide.as_ptr()),
                PSID(ptr::null_mut()),
                &mut sid_size,
                PWSTR(ptr::null_mut()),
                &mut domain_name_size,
                &mut sid_name_use as *mut _,
            );

            // Check if the function call failed due to insufficient buffer
            if lookup_result.is_ok() || GetLastError().0 != 122 {
                // 122 is ERROR_INSUFFICIENT_BUFFER
                tracing::error!(
                    "{:?} -> LookupAccountNameW failed to get buffer sizes. Error code: {}",
                    id,
                    GetLastError().0
                );
                return Err(anyhow::Error::msg(format!(
                    "LookupAccountNameW failed to get buffer sizes. Error code: {}",
                    GetLastError().0
                )));
            }

            let mut sid_buffer = vec![0u8; sid_size as usize];
            let sid = PSID(sid_buffer.as_mut_ptr() as *mut _);

            let mut domain_name_buffer = vec![0u16; domain_name_size as usize];

            // Second call to LookupAccountNameW to get the actual data
            let lookup_result = LookupAccountNameW(
                PCWSTR(ptr::null()),
                PCWSTR(user_name_wide.as_ptr()),
                sid,
                &mut sid_size,
                PWSTR(domain_name_buffer.as_mut_ptr()),
                &mut domain_name_size,
                &mut sid_name_use as *mut _,
            );

            // Corrected condition: Treat failure as error
            if lookup_result.is_err() {
                let error_code = GetLastError();
                tracing::error!(
                    "{:?} -> LookupAccountNameW failed. Error code: {}",
                    id,
                    error_code.0
                );
                return Err(anyhow::Error::msg(format!(
                    "LookupAccountNameW failed. Error code: {}",
                    error_code.0
                )));
            }

            // Prepare to enumerate account rights
            let mut rights_ptr: *mut LSA_UNICODE_STRING = ptr::null_mut();
            let mut rights_count: u32 = 0;

            // Call LsaEnumerateAccountRights to get the rights assigned to the SID
            let status =
                LsaEnumerateAccountRights(policy_handle, sid, &mut rights_ptr, &mut rights_count);

            if status == NTSTATUS(0) {
                // Create a slice from the returned rights
                let rights_slice = std::slice::from_raw_parts(rights_ptr, rights_count as usize);

                let privilege_wide: Vec<u16> = self.key.encode_utf16().collect();

                // Check if the privilege is present in the user's rights
                let has_privilege = rights_slice.iter().any(|right| {
                    let right_str =
                        std::slice::from_raw_parts(right.Buffer.0, (right.Length / 2) as usize);
                    right_str == privilege_wide.as_slice()
                });

                // Free the memory allocated by LsaEnumerateAccountRights
                let free_status = LsaFreeMemory(Some(rights_ptr as *mut _));
                if free_status != NTSTATUS(0) {
                    tracing::error!(
                        "LsaFreeMemory failed with error code: {}",
                        LsaNtStatusToWinError(free_status)
                    );
                }

                if has_privilege {
                    tracing::info!(
                        "{:?} -> Group Policy tweak is enabled for user '{}'.",
                        id,
                        user_name
                    );
                    Ok(GroupPolicyValue::Enabled)
                } else {
                    tracing::info!(
                        "{:?} -> Group Policy tweak is disabled for user '{}'.",
                        id,
                        user_name
                    );
                    Ok(GroupPolicyValue::Disabled)
                }
            } else if status == STATUS_OBJECT_NAME_NOT_FOUND {
                // The account has no rights assigned
                tracing::info!(
                    "{:?} -> Group Policy tweak is disabled for user '{}'.",
                    id,
                    user_name
                );
                Ok(GroupPolicyValue::Disabled)
            } else {
                let win_err = LsaNtStatusToWinError(status);
                tracing::error!(
                    "{:?} -> LsaEnumerateAccountRights failed with error code: {}",
                    id,
                    win_err
                );
                Err(anyhow::Error::msg(format!(
                    "LsaEnumerateAccountRights failed with error code: {}",
                    win_err
                )))
            }
        }
    }

    /// Modifies user rights by adding or removing a specified privilege.
    ///
    /// # Parameters
    ///
    /// - `privilege`: The privilege to add or remove.
    /// - `enable`: If `true`, adds the privilege; if `false`, removes it.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(GroupPolicyError)` if the operation fails.
    fn modify_user_rights(&self, privilege: &str, enable: bool) -> Result<(), anyhow::Error> {
        unsafe {
            let object_attributes = LSA_OBJECT_ATTRIBUTES::default();

            let mut policy_handle: LSA_HANDLE = LSA_HANDLE(0);

            let desired_access = POLICY_CREATE_ACCOUNT | POLICY_LOOKUP_NAMES;

            // Open the policy with the desired access
            let status =
                LsaOpenPolicy(None, &object_attributes, desired_access, &mut policy_handle);
            if status != NTSTATUS(0) {
                let win_err = LsaNtStatusToWinError(status);
                tracing::error!("LsaOpenPolicy failed with error code: {}", win_err);
                return Err(anyhow::Error::msg(format!(
                    "LsaOpenPolicy failed with error code: {}",
                    win_err
                )));
            }

            // Ensure the policy handle is closed properly
            let _policy_guard = LsaHandleGuard {
                handle: policy_handle,
            };

            let mut sid_size = 0u32;
            let mut domain_name_size = 0u32;
            let mut sid_name_use = SID_NAME_USE(0);

            let user_name = whoami::username();
            let user_name_wide: Vec<u16> = user_name.encode_utf16().chain(Some(0)).collect();

            // First call to LookupAccountNameW to get buffer sizes
            let _ = LookupAccountNameW(
                PCWSTR(ptr::null()),
                PCWSTR(user_name_wide.as_ptr()),
                PSID(ptr::null_mut()),
                &mut sid_size,
                PWSTR(ptr::null_mut()),
                &mut domain_name_size,
                &mut sid_name_use as *mut _,
            );

            let mut sid_buffer = vec![0u8; sid_size as usize];
            let sid = PSID(sid_buffer.as_mut_ptr() as *mut _);

            let mut domain_name_buffer = vec![0u16; domain_name_size as usize];

            // Second call to LookupAccountNameW to get the actual data
            if LookupAccountNameW(
                PCWSTR(ptr::null()),
                PCWSTR(user_name_wide.as_ptr()),
                sid,
                &mut sid_size,
                PWSTR(domain_name_buffer.as_mut_ptr()),
                &mut domain_name_size,
                &mut sid_name_use as *mut _,
            )
            .is_ok()
            {
                let privilege_wide: Vec<u16> = privilege.encode_utf16().collect();

                let privilege_lsa_string = LSA_UNICODE_STRING {
                    Length: (privilege_wide.len() * 2) as u16,
                    MaximumLength: (privilege_wide.len() * 2) as u16,
                    Buffer: PWSTR(privilege_wide.as_ptr() as *mut _),
                };

                let user_rights = [privilege_lsa_string];

                if enable {
                    // Add the privilege to the user
                    let status = LsaAddAccountRights(policy_handle, sid, &user_rights);
                    if status != NTSTATUS(0) {
                        let win_err = LsaNtStatusToWinError(status);
                        tracing::error!("LsaAddAccountRights failed with error code: {}", win_err);
                        return Err(anyhow::Error::msg(format!(
                            "LsaAddAccountRights failed with error code: {}",
                            win_err
                        )));
                    }
                    tracing::info!(
                        "Successfully added privilege '{}' to user '{}'.",
                        privilege,
                        user_name
                    );
                } else {
                    // Remove the privilege from the user
                    let status =
                        LsaRemoveAccountRights(policy_handle, sid, false, Some(&user_rights));
                    if status != NTSTATUS(0) {
                        let win_err = LsaNtStatusToWinError(status);
                        // Treat error code 2 (ERROR_FILE_NOT_FOUND) as success
                        if win_err != 2 {
                            tracing::error!(
                                "LsaRemoveAccountRights failed with error code: {}",
                                win_err
                            );
                            return Err(anyhow::Error::msg(format!(
                                "LsaRemoveAccountRights failed with error code: {}",
                                win_err
                            )));
                        } else {
                            tracing::debug!(
                                "Privilege '{}' was not assigned to user '{}'; nothing to remove.",
                                privilege,
                                user_name
                            );
                        }
                    } else {
                        tracing::info!(
                            "Successfully removed privilege '{}' from user '{}'.",
                            privilege,
                            user_name
                        );
                    }
                }

                Ok(())
            } else {
                let error_code = GetLastError();
                tracing::error!("LookupAccountNameW failed. Error code: {}", error_code.0);
                Err(anyhow::Error::msg(format!(
                    "LookupAccountNameW failed. Error code: {}",
                    error_code.0
                )))
            }
        }
    }
}

impl TweakMethod for GroupPolicyTweak {
    /// Checks if the tweak is currently enabled by comparing the current value to the desired value.
    ///
    /// # Returns
    /// - `Ok(true)` if the tweak is enabled.
    /// - `Ok(false)` if the tweak is disabled.
    /// - `Err(GroupPolicyError)` if the operation fails.
    fn initial_state(&self, id: TweakId) -> Result<bool, anyhow::Error> {
        tracing::info!("{:?} -> Checking if Group Policy tweak is enabled.", id);
        match self.read_current_value(id) {
            Ok(value) => {
                let is_enabled = value == self.value;
                tracing::info!(
                    "{:?} -> Group Policy tweak is {}.",
                    id,
                    if is_enabled { "enabled" } else { "disabled" }
                );
                Ok(is_enabled)
            }
            Err(e) => {
                tracing::error!(
                    "{:?} -> Failed to check if Group Policy tweak is enabled: {:?}",
                    id,
                    e
                );
                Err(e)
            }
        }
    }

    /// Applies the Group Policy tweak by assigning the specified privilege to the current user.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(GroupPolicyError)` if the operation fails.
    fn apply(&self, id: TweakId) -> Result<(), anyhow::Error> {
        tracing::info!("{:?} -> Applying Group Policy tweak.", id);
        // Assign the privilege to the current user
        match self.modify_user_rights(&self.key, true) {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("{:?} -> Failed to apply Group Policy tweak: {:?}", id, e);
                Err(e)
            }
        }
    }

    /// Reverts the Group Policy tweak by removing the specified privilege from the current user.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(GroupPolicyError)` if the operation fails.
    fn revert(&self, id: TweakId) -> Result<(), anyhow::Error> {
        tracing::info!("{:?} -> Reverting Group Policy tweak.", id);
        // Remove the privilege from the current user
        match self.modify_user_rights(&self.key, false) {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("{:?} -> Failed to revert Group Policy tweak: {:?}", id, e);
                Err(e)
            }
        }
    }
}

pub struct LsaHandleGuard {
    pub handle: LSA_HANDLE,
}

impl Drop for LsaHandleGuard {
    fn drop(&mut self) {
        unsafe {
            let status = LsaClose(self.handle);
            if status != NTSTATUS(0) {
                tracing::error!(
                    "LsaClose failed with error code: {}",
                    LsaNtStatusToWinError(status)
                );
            } else {
                tracing::debug!("Successfully closed LSA_HANDLE.");
            }
        }
    }
}

pub fn se_lock_memory_privilege() -> Arc<Mutex<Tweak>> {
    Tweak::group_policy(
        "SeLockMemoryPrivilege".to_string(),
        "The SeLockMemoryPrivilege group policy setting allows a process to lock pages in physical memory, preventing them from being paged out to disk. This can improve performance for applications that require fast, consistent access to critical data by keeping it always available in RAM.".to_string(),
        TweakCategory::Memory,
        GroupPolicyTweak {
            key: "SeLockMemoryPrivilege".to_string(),
            value: GroupPolicyValue::Enabled,
        },
        true,
    )
}
