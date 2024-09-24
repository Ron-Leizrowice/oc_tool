// src/main.rs

mod actions;
mod errors;
mod models;
mod tweaks;
mod ui;
mod utils;
use std::{process::Command, ptr};

use druid::{
    im::Vector,
    widget::{Controller, CrossAxisAlignment, Flex, Label, List, Scroll, Switch},
    AppLauncher, Data, Env, Event, EventCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString,
    UpdateCtx, Widget, WidgetExt, WindowDesc,
};
use once_cell::sync::Lazy;
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
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    RegKey,
};

use crate::errors::{GroupPolicyTweakError, RegistryTweakError};

const POLICY_CREATE_ACCOUNT: u32 = 0x00000010;
const POLICY_LOOKUP_NAMES: u32 = 0x00000800;

pub fn build_root_widget() -> impl Widget<AppState> {
    let list = List::new(make_tweak_switch);
    let scroll = Scroll::new(list)
        .vertical()
        .padding(10.0)
        .expand_height()
        .lens(AppState::tweak_list);

    let info_bar = Label::new(|data: &AppState, _: &_| {
        let count = data
            .tweak_list
            .iter()
            .filter(|tweak| tweak.enabled && tweak.requires_restart)
            .count();
        if count > 0 {
            format!("{} tweaks pending restart", count)
        } else {
            "".to_string()
        }
    })
    .padding(5.0);

    Flex::column()
        .with_flex_child(scroll, 1.0)
        .with_child(info_bar)
}

// Helper function to generate a new switch widget for a tweak
fn make_tweak_switch() -> impl Widget<Tweak> {
    let label = Label::new(|data: &Tweak, _: &_| data.name.clone())
        .fix_width(250.0)
        .padding(5.0);

    let switch = Switch::new().lens(Tweak::enabled);

    Flex::row()
        .with_child(label)
        .with_flex_spacer(1.0)
        .with_child(switch)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .controller(TweakController)
}

// base model for any tweak, all subtypes will be implemented as traits for this model
#[derive(Clone, Data, Lens)]
pub struct Tweak {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub requires_restart: bool,
    pub config: TweakMethod,
}

// Trait defining the apply and revert methods
pub trait TweakAction {
    fn read(&self) -> Result<(), anyhow::Error>;
    fn apply(&self) -> Result<(), anyhow::Error>;
    fn revert(&self) -> Result<(), anyhow::Error>;
}

// Implement TweakAction for Tweak
impl TweakAction for Tweak {
    fn read(&self) -> Result<(), anyhow::Error> {
        match &self.config {
            TweakMethod::Registry(config) => {
                config.read_current_value()?;
            }
            TweakMethod::GroupPolicy(config) => {
                config.read_current_value()?;
            }
        }
        Ok(())
    }

    fn apply(&self) -> Result<(), anyhow::Error> {
        match &self.config {
            TweakMethod::Registry(config) => {
                config.apply_registry_tweak()?;
            }
            TweakMethod::GroupPolicy(config) => {
                config.apply_group_policy_tweak()?;
            }
        }
        Ok(())
    }

    fn revert(&self) -> Result<(), anyhow::Error> {
        match &self.config {
            TweakMethod::Registry(config) => {
                config.revert_registry_tweak()?;
            }
            TweakMethod::GroupPolicy(config) => {
                config.revert_group_policy_tweak()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Data)]
pub enum TweakMethod {
    Registry(RegistryTweak),
    GroupPolicy(GroupPolicyTweak),
}

// Subclass for Registry tweaks
#[derive(Clone, Data, Lens, Debug)]
pub struct RegistryTweak {
    pub key: String,
    pub name: String,
    pub value: RegistryKeyValue,
    pub default_value: RegistryKeyValue,
}

#[derive(Clone, Data, PartialEq, Eq, Debug)]
pub enum RegistryKeyValue {
    String(String),
    Dword(u32),
    // Add other types as needed (e.g., Qword, Binary, etc.)
}

impl RegistryTweak {
    // Function to read the current registry value
    pub fn read_current_value(&self) -> Result<RegistryKeyValue, RegistryTweakError> {
        // Extract the hive from the key path
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryTweakError::UnsupportedHive(other.to_string())),
        };

        // Extract the subkey path (everything after the hive)
        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        // Attempt to open the subkey with read permissions
        let subkey = hkey
            .open_subkey_with_flags(subkey_path, KEY_READ)
            .map_err(|e| {
                RegistryTweakError::KeyOpenError(format!(
                    "Failed to open registry key '{}': {}",
                    self.key, e
                ))
            })?;

        // Depending on the expected type, read the value
        match &self.value {
            RegistryKeyValue::String(_) => {
                let val: String = subkey.get_value(&self.name).map_err(|e| {
                    RegistryTweakError::ReadValueError(format!(
                        "Failed to read string value '{:.?}': {:.?}",
                        self.value, e
                    ))
                })?;
                Ok(RegistryKeyValue::String(val))
            }
            RegistryKeyValue::Dword(_) => {
                let val: u32 = subkey.get_value(&self.name).map_err(|e| {
                    RegistryTweakError::ReadValueError(format!(
                        "Failed to read DWORD value '{:.?}': {:.?}",
                        self.value, e
                    ))
                })?;
                Ok(RegistryKeyValue::Dword(val))
            } // Handle other types as needed
        }
    }

    pub fn apply_registry_tweak(&self) -> Result<(), RegistryTweakError> {
        // Extract the hive from the key path
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        // Map the hive string to the corresponding RegKey
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryTweakError::UnsupportedHive(other.to_string())),
        };

        // Extract the subkey path (everything after the hive)
        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        // Attempt to open the subkey with read and write permissions
        // If it doesn't exist, create it
        let subkey = match hkey.open_subkey_with_flags(subkey_path, KEY_READ | KEY_WRITE) {
            Ok(key) => key, // Subkey exists and is opened successfully
            Err(_) => {
                // Subkey does not exist; attempt to create it
                match hkey.create_subkey(subkey_path) {
                    Ok((key, disposition)) => {
                        // Log whether the key was created or already existed
                        match disposition {
                            winreg::enums::RegDisposition::REG_CREATED_NEW_KEY => {
                                println!("Created new registry key: {}", self.key);
                            }
                            winreg::enums::RegDisposition::REG_OPENED_EXISTING_KEY => {
                                println!("Opened existing registry key: {}", self.key);
                            }
                        }
                        key
                    }
                    Err(e) => {
                        return Err(RegistryTweakError::CreateError(format!(
                            "Failed to create registry key '{:?}': {:?}",
                            self.key, e
                        )))
                    }
                }
            }
        };

        // Now, set the registry value based on its type
        match &self.value {
            RegistryKeyValue::String(val) => {
                subkey.set_value(&self.name, val).map_err(|e| {
                    RegistryTweakError::SetValueError(format!(
                        "Failed to set string value '{:?}': {:?}",
                        self.value, e
                    ))
                })?;
                println!(
                    "Set string value '{:.?}' to '{:.?}' in key '{:.?}'",
                    self.value, val, self.key
                );
            }
            RegistryKeyValue::Dword(val) => {
                subkey.set_value(&self.name, val).map_err(|e| {
                    RegistryTweakError::SetValueError(format!(
                        "Failed to set DWORD value '{:.?}': {:.?}",
                        self.value, e
                    ))
                })?;
                println!(
                    "Set DWORD value '{:.?}' to '{:.?}' in key '{:.?}'",
                    self.value, val, self.key
                );
            } // Handle other types as needed
        }

        Ok(())
    }

    pub fn revert_registry_tweak(&self) -> Result<(), RegistryTweakError> {
        let hive = self
            .key
            .split('\\')
            .next()
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;
        let hkey = match hive {
            "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
            other => return Err(RegistryTweakError::UnsupportedHive(other.to_string())),
        };

        let subkey_path = self
            .key
            .split_once('\\')
            .map(|(_, path)| path)
            .ok_or_else(|| RegistryTweakError::InvalidKeyFormat(self.key.clone()))?;

        let subkey = hkey
            .open_subkey_with_flags(subkey_path, KEY_WRITE)
            .map_err(|e| {
                RegistryTweakError::KeyOpenError(format!(
                    "Failed to open registry key '{}': {}",
                    self.key, e
                ))
            })?;

        match &self.default_value {
            RegistryKeyValue::String(val) => subkey
                .set_value(self.name.clone(), val)
                .map_err(|e| RegistryTweakError::SetValueError(e.to_string())),
            RegistryKeyValue::Dword(val) => subkey
                .set_value(self.name.clone(), val)
                .map_err(|e| RegistryTweakError::SetValueError(e.to_string())),
            // Handle other types as needed
        }
    }
}

// Subclass for Group Policy tweaks
#[derive(Clone, Data, Lens, Debug)]
pub struct GroupPolicyTweak {
    pub key: String,
    pub value: Option<String>,
}

pub enum GroupPolicyValue {
    Enabled,
    Disabled,
}

impl GroupPolicyTweak {
    pub fn read_current_value(&self) -> Result<GroupPolicyValue, GroupPolicyTweakError> {
        unsafe {
            let object_attributes = LSA_OBJECT_ATTRIBUTES::default();

            let mut policy_handle: LSA_HANDLE = LSA_HANDLE(0);

            let desired_access = POLICY_LOOKUP_NAMES;

            let status =
                LsaOpenPolicy(None, &object_attributes, desired_access, &mut policy_handle);
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
                let mut rights_ptr: *mut LSA_UNICODE_STRING = ptr::null_mut();
                let mut rights_count: u32 = 0;

                let status = LsaEnumerateAccountRights(
                    policy_handle,
                    sid,
                    &mut rights_ptr,
                    &mut rights_count,
                );

                if status == NTSTATUS(0) {
                    let rights_slice =
                        std::slice::from_raw_parts(rights_ptr, rights_count as usize);

                    let privilege_wide: Vec<u16> = self.key.encode_utf16().collect();

                    let has_privilege = rights_slice.iter().any(|right| {
                        let right_str =
                            std::slice::from_raw_parts(right.Buffer.0, (right.Length / 2) as usize);
                        right_str == privilege_wide.as_slice()
                    });

                    // Free the memory allocated by LsaEnumerateAccountRights
                    LsaFreeMemory(Some(rights_ptr as *mut _));

                    match has_privilege {
                        true => Ok(GroupPolicyValue::Enabled),
                        false => Ok(GroupPolicyValue::Disabled),
                    }
                } else if status == STATUS_OBJECT_NAME_NOT_FOUND {
                    // The account has no rights assigned
                    match self.value {
                        Some(_) => Ok(GroupPolicyValue::Disabled),
                        None => Ok(GroupPolicyValue::Enabled),
                    }
                } else {
                    let win_err = LsaNtStatusToWinError(status);
                    Err(GroupPolicyTweakError::ReadValueError(format!(
                        "LsaEnumerateAccountRights failed with error code: {}",
                        win_err
                    )))
                }
            } else {
                let error_code = GetLastError();
                Err(GroupPolicyTweakError::KeyOpenError(format!(
                    "LookupAccountNameW failed. Error code: {}",
                    error_code.0
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

            let status =
                LsaOpenPolicy(None, &object_attributes, desired_access, &mut policy_handle);
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
                // Run gpupdate /force after applying the tweak
                Command::new("gpupdate")
                    .args(&["/force"])
                    .status()
                    .expect("Failed to execute gpupdate");
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

struct LsaHandleGuard {
    handle: LSA_HANDLE,
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

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub tweak_list: Vector<Tweak>,
}

impl Default for AppState {
    fn default() -> Self {
        let tweaks = Lazy::new(|| {
            Vector::from(vec![
                LARGE_SYSTEM_CACHE.clone(),
                SE_LOCK_MEMORY_PRIVILEGE.clone(),
            ])
        });

        let mut updated_tweaks = Vector::new();

        for mut tweak in tweaks.iter().cloned() {
            // Initialize 'enabled' based on current system settings
            match &tweak.config {
                TweakMethod::Registry(registry_tweak) => {
                    match registry_tweak.read_current_value() {
                        Ok(current_value) => {
                            // Compare current value with desired value
                            if current_value == registry_tweak.value {
                                tweak.enabled = true;
                            } else {
                                tweak.enabled = false;
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to read current value for registry tweak '{}': {}",
                                tweak.name, e
                            );
                            tweak.enabled = false; // Default to disabled on error
                        }
                    }
                }
                TweakMethod::GroupPolicy(gp_tweak) => match gp_tweak.read_current_value() {
                    Ok(GroupPolicyValue::Enabled) => tweak.enabled = true,
                    Ok(GroupPolicyValue::Disabled) => tweak.enabled = false,
                    Err(e) => {
                        eprintln!(
                            "Failed to read current value for group policy tweak '{}': {}",
                            tweak.name, e
                        );
                        tweak.enabled = false;
                    }
                },
            }
            updated_tweaks.push_back(tweak);
        }

        AppState {
            tweak_list: updated_tweaks,
        }
    }
}

// Controller to handle apply and revert actions
pub struct TweakController;

impl<W: Widget<Tweak>> Controller<Tweak, W> for TweakController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Tweak,
        env: &Env,
    ) {
        child.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Tweak,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &Tweak,
        data: &Tweak,
        env: &Env,
    ) {
        if old_data.enabled != data.enabled {
            if data.enabled {
                if let Err(e) = data.apply() {
                    println!("Failed to apply tweak '{}': {}", data.name, e);
                } else {
                    println!("Applied tweak '{}'", data.name);
                }
            } else {
                if let Err(e) = data.revert() {
                    println!("Failed to revert tweak '{}': {}", data.name, e);
                } else {
                    println!("Reverted tweak '{}'", data.name);
                }
            }
        }
        child.update(ctx, old_data, data, env);
    }
}

pub static LARGE_SYSTEM_CACHE: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
    name: "LargeSystemCache".to_string(),
    enabled: false,
    description: "Optimizes system memory management by adjusting the LargeSystemCache setting.".to_string(),
    config: TweakMethod::Registry( RegistryTweak{
        key: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
        name: "LargeSystemCache".to_string(),
        value: RegistryKeyValue::Dword(1),
        default_value: RegistryKeyValue::Dword(0),
    }),
    requires_restart: false,
}
});

pub static SE_LOCK_MEMORY_PRIVILEGE: Lazy<Tweak> = Lazy::new(|| Tweak {
    name: "SeLockMemoryPrivilege".to_string(),
    enabled: false,
    description: "Assigns the 'Lock pages in memory' privilege to the current user.".to_string(),
    config: TweakMethod::GroupPolicy(GroupPolicyTweak {
        key: "SeLockMemoryPrivilege".to_string(),
        value: None,
    }),
    requires_restart: true,
});

fn main() {
    // Setup the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title(LocalizedString::new("OC Tool"))
        .window_size((400.0, 400.0));

    // Create the initial app state with multiple tweaks
    let initial_state = AppState::default();

    // Start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("launch failed");
}
