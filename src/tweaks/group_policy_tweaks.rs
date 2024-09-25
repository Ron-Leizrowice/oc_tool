// src/tweaks/group_policy_tweaks.rs

use std::ptr;

use druid::{Data, Lens};
use once_cell::sync::Lazy;
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{GetLastError, NTSTATUS, STATUS_OBJECT_NAME_NOT_FOUND},
        Security::{
            Authentication::Identity::{
                LsaAddAccountRights, LsaEnumerateAccountRights, LsaFreeMemory,
                LsaNtStatusToWinError, LsaOpenPolicy, LsaRemoveAccountRights, LSA_HANDLE,
                LSA_OBJECT_ATTRIBUTES, LSA_UNICODE_STRING, POLICY_CREATE_ACCOUNT,
                POLICY_LOOKUP_NAMES,
            },
            LookupAccountNameW, PSID, SID_NAME_USE,
        },
    },
};

use super::TweakMethod;
use crate::{
    errors::GroupPolicyTweakError, models::Tweak, ui::widgets::WidgetType, utils::LsaHandleGuard,
};

// Subclass for Group Policy tweaks
#[derive(Clone, Data, Lens, Debug)]
pub struct GroupPolicyTweak {
    pub key: String,
    pub value: GroupPolicyValue,
    pub default_value: GroupPolicyValue,
}

#[derive(Clone, Copy, Data, PartialEq, Eq, Debug)]
pub enum GroupPolicyValue {
    Enabled,
    Disabled,
}

impl GroupPolicyTweak {
    pub fn read_current_value(&self) -> Result<GroupPolicyValue, GroupPolicyTweakError> {
        unsafe {
            // Initialize object attributes for LsaOpenPolicy
            let object_attributes = LSA_OBJECT_ATTRIBUTES::default();
            // Initialize the policy handle to zero to avoid using uninitialized memory
            let mut policy_handle = LSA_HANDLE(0);
            // Define the desired access rights for the policy object handle (read-only)
            let desired_access = POLICY_LOOKUP_NAMES;
            // Call LsaOpenPolicy to get a handle to the policy object
            let status = LsaOpenPolicy(
                None,
                &object_attributes,
                desired_access as u32,
                &mut policy_handle,
            );
            // Check the return value of LsaOpenPolicy
            if status != NTSTATUS(0) {
                let win_err = LsaNtStatusToWinError(status);
                return Err(GroupPolicyTweakError::KeyOpenError(format!(
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

            let user_name_wide: Vec<u16> =
                whoami::username().encode_utf16().chain(Some(0)).collect();

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
                return Err(GroupPolicyTweakError::KeyOpenError(format!(
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

            // Check the return value of LookupAccountNameW
            if !lookup_result.is_ok() {
                let error_code = GetLastError();
                return Err(GroupPolicyTweakError::KeyOpenError(format!(
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

                let has_privilege = rights_slice.iter().any(|right| {
                    let right_str =
                        std::slice::from_raw_parts(right.Buffer.0, (right.Length / 2) as usize);
                    right_str == privilege_wide.as_slice()
                });

                // Free the memory allocated by LsaEnumerateAccountRights
                let free_status = LsaFreeMemory(Some(rights_ptr as *mut _));
                if free_status != NTSTATUS(0) {
                    eprintln!(
                        "LsaFreeMemory failed with error code: {}",
                        LsaNtStatusToWinError(free_status)
                    );
                }

                if has_privilege {
                    Ok(GroupPolicyValue::Enabled)
                } else {
                    Ok(GroupPolicyValue::Disabled)
                }
            } else if status == STATUS_OBJECT_NAME_NOT_FOUND {
                // The account has no rights assigned
                Ok(GroupPolicyValue::Disabled)
            } else {
                let win_err = LsaNtStatusToWinError(status);
                Err(GroupPolicyTweakError::ReadValueError(format!(
                    "LsaEnumerateAccountRights failed with error code: {}",
                    win_err
                )))
            }
        }
    }

    pub fn apply_group_policy_tweak(&self) -> Result<(), GroupPolicyTweakError> {
        // Assign the privilege to the current user
        self.modify_user_rights(&self.key, true)
    }

    pub fn revert_group_policy_tweak(&self) -> Result<(), GroupPolicyTweakError> {
        // Remove the privilege from the current user
        self.modify_user_rights(&self.key, false)
    }

    fn modify_user_rights(
        &self,
        privilege: &str,
        enable: bool,
    ) -> Result<(), GroupPolicyTweakError> {
        unsafe {
            let object_attributes = LSA_OBJECT_ATTRIBUTES::default();

            let mut policy_handle: LSA_HANDLE = LSA_HANDLE(0);

            let desired_access = POLICY_CREATE_ACCOUNT | POLICY_LOOKUP_NAMES;

            let status = LsaOpenPolicy(
                None,
                &object_attributes,
                desired_access as u32,
                &mut policy_handle,
            );
            if status != NTSTATUS(0) {
                let win_err = LsaNtStatusToWinError(status);
                return Err(GroupPolicyTweakError::KeyOpenError(format!(
                    "LsaOpenPolicy failed with error code: {}",
                    win_err
                )));
            }

            let _policy_guard = LsaHandleGuard {
                handle: policy_handle,
            };

            let mut sid_size = 0u32;
            let mut domain_name_size = 0u32;
            let mut sid_name_use = SID_NAME_USE(0);

            let user_name = whoami::username();
            println!("Current user: {}", user_name);
            let user_name_wide: Vec<u16> = user_name.encode_utf16().chain(Some(0)).collect();

            // First call to get buffer sizes
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

            // Second call to get actual data
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
                    let status = LsaAddAccountRights(policy_handle, sid, &user_rights);
                    if status != NTSTATUS(0) {
                        let win_err = LsaNtStatusToWinError(status);
                        return Err(GroupPolicyTweakError::SetValueError(format!(
                            "LsaAddAccountRights failed with error code: {}",
                            win_err
                        )));
                    }
                } else {
                    let status =
                        LsaRemoveAccountRights(policy_handle, sid, false, Some(&user_rights));
                    if status != NTSTATUS(0) {
                        let win_err = LsaNtStatusToWinError(status);
                        // Treat error code 2 (ERROR_FILE_NOT_FOUND) as success
                        if win_err != 2 {
                            return Err(GroupPolicyTweakError::SetValueError(format!(
                                "LsaRemoveAccountRights failed with error code: {}",
                                win_err
                            )));
                        } else {
                            // Privilege was not assigned, so we can consider it already removed
                            println!(
                                "Privilege '{}' was not assigned to the user; nothing to remove.",
                                privilege
                            );
                        }
                    }
                }

                Ok(())
            } else {
                let error_code = GetLastError();
                Err(GroupPolicyTweakError::KeyOpenError(format!(
                    "LookupAccountNameW failed. Error code: {}",
                    error_code.0
                )))
            }
        }
    }
}

pub static SE_LOCK_MEMORY_PRIVILEGE: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "SeLockMemoryPrivilege".to_string(),
    description: "Assigns the 'Lock pages in memory' privilege to the current user.".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::GroupPolicy(GroupPolicyTweak {
        key: "SeLockMemoryPrivilege".to_string(),
        value: GroupPolicyValue::Enabled,
        default_value: GroupPolicyValue::Disabled,
    }),
    requires_restart: true,
    applying: false,
});
