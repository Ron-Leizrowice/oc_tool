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
    widget::{Button, Controller, CrossAxisAlignment, Either, Flex, Label, List, Scroll, Switch},
    AppLauncher, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LocalizedString, PaintCtx, Selector, Size, Target, UpdateCtx, Widget, WidgetExt,
    WindowDesc,
};
use once_cell::sync::Lazy;
use tweaks::ALL_TWEAKS;
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

static USERNAME: Lazy<String> = Lazy::new(|| whoami::username());

const POLICY_CREATE_ACCOUNT: u32 = 0x00000010;
const POLICY_LOOKUP_NAMES: u32 = 0x00000800;

const SET_APPLYING: Selector<(usize, bool)> = Selector::new("my_app.set_applying");
const UPDATE_TWEAK_ENABLED: Selector<(usize, bool)> = Selector::new("my_app.update_tweak_enabled");

pub fn build_root_widget() -> impl Widget<AppState> {
    let list = List::new(make_tweak_widget);
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

fn make_tweak_widget() -> impl Widget<Tweak> {
    // Common label for all tweaks
    let label = Label::new(|data: &Tweak, _: &_| data.name.clone())
        .fix_width(250.0)
        .padding(5.0);

    // Placeholder for the control widget (Switch or Button)
    let control = Either::new(
        |data: &Tweak, _: &_| data.widget == WidgetType::Switch,
        make_switch(),
        make_command_button(),
    );

    let applying_label = Label::new(|data: &Tweak, _: &_| {
        if data.applying {
            "applying".to_string()
        } else {
            "".to_string()
        }
    })
    .fix_width(70.0) // Set fixed width to prevent layout shift
    .padding(5.0);

    Flex::row()
        .with_child(label)
        .with_flex_spacer(1.0)
        .with_child(control)
        .with_child(applying_label)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .controller(TweakController::new())
}

fn make_switch() -> impl Widget<Tweak> {
    TweakSwitch {
        child: Switch::new(),
    }
}

fn make_command_button() -> impl Widget<Tweak> {
    Button::new("Apply")
        .on_click(|ctx, data: &mut Tweak, _env| {
            if data.applying {
                return;
            }
            data.applying = true;
            ctx.request_paint();

            let sink = ctx.get_external_handle();
            let tweak_id = data.id;
            let data_clone = data.clone();

            std::thread::spawn(move || {
                let result = data_clone.apply();

                if let Err(ref e) = result {
                    println!("Failed to execute command '{}': {}", data_clone.name, e);
                } else {
                    println!("Executed command '{}'", data_clone.name);
                }

                sink.submit_command(SET_APPLYING, (tweak_id, false), Target::Auto)
                    .expect("Failed to submit command");
            });
        })
        .controller(ButtonController)
}

// Base model for any tweak, all subtypes will be implemented as traits for this model
#[derive(Clone, Data, Lens)]
pub struct Tweak {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub widget: WidgetType,
    pub enabled: bool,
    pub requires_restart: bool,
    pub applying: bool,
    pub config: TweakMethod,
}

#[derive(Clone, Data, PartialEq)]
pub enum WidgetType {
    Switch,
    Button,
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
            TweakMethod::Command(_) => {
                // For CommandTweaks, read can be a no-op
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
            TweakMethod::Command(config) => {
                config.apply()?;
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
            TweakMethod::Command(_) => {
                // Typically, commands cannot be reverted, so you can leave this empty or return an error
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Data)]
pub enum TweakMethod {
    Registry(RegistryTweak),
    GroupPolicy(GroupPolicyTweak),
    Command(CommandTweak),
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
            }
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
                        "Failed to set string value '{:?}' in key '{:?}': {:?}",
                        self.name, self.key, e
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
                        "Failed to set DWORD value '{:?}' in key '{:?}': {:?}",
                        self.name, self.key, e
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
                .set_value(&self.name, val)
                .map_err(|e| RegistryTweakError::SetValueError(e.to_string())),
            RegistryKeyValue::Dword(val) => subkey
                .set_value(&self.name, val)
                .map_err(|e| RegistryTweakError::SetValueError(e.to_string())),
            // Handle other types as needed
        }
    }
}

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
            let mut policy_handle: LSA_HANDLE = LSA_HANDLE(0);
            // Define the desired access rights for the policy object handle (read-only)
            let desired_access = POLICY_LOOKUP_NAMES;
            // Call LsaOpenPolicy to get a handle to the policy object
            let status =
                LsaOpenPolicy(None, &object_attributes, desired_access, &mut policy_handle);
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

            let user_name_wide: Vec<u16> = USERNAME.encode_utf16().chain(Some(0)).collect();

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

#[derive(Clone, Data, Lens, Debug)]
pub struct CommandTweak {
    pub read_commands: Option<Vector<String>>,
    pub apply_commands: Vector<String>,
    pub revert_commands: Option<Vector<String>>,
    pub target_state: Option<Vector<String>>,
}

impl CommandTweak {
    pub fn is_enabled(&self) -> bool {
        // For CommandTweaks, attempt to read the current state, and compare with the default state
        match self.target_state {
            Some(ref target_state) => {
                let current_state = self.read_current_state().unwrap_or(None);
                current_state == Some(target_state.clone().into_iter().collect())
            }
            None => false,
        }
    }

    pub fn read_current_state(&self) -> Result<Option<Vec<String>>, anyhow::Error> {
        // For CommandTweak, read can be a no-op or return an appropriate state
        match &self.read_commands {
            Some(commands) => {
                let output = commands.iter().map(|c| {
                    Command::new("cmd")
                        .args(&["/C", c])
                        .output()
                        .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", c, e))
                });

                let results: Result<Vec<String>, anyhow::Error> = output
                    .map(|res| {
                        res.and_then(|output| {
                            String::from_utf8(output.stdout).map_err(|e| {
                                anyhow::anyhow!("Failed to convert output to string: {}", e)
                            })
                        })
                    })
                    .collect();
                Ok(Some(results?))
            }
            None => Ok(None),
        }
    }

    pub fn apply(&self) -> Result<(), anyhow::Error> {
        let result = self.apply_commands.iter().try_for_each(|c| {
            let output = Command::new("cmd")
                .args(&["/C", c])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", c, e))?;

            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!(
                    "Command '{}' failed with error: {}",
                    c,
                    stderr
                ))
            }
        });

        match result {
            Ok(_) => {
                println!("Successfully applied the tweak");
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to apply the tweak: {}", e);
                Err(e)
            }
        }
    }

    pub fn revert(&self) -> Result<(), anyhow::Error> {
        if let Some(revert_commands) = &self.revert_commands {
            revert_commands.iter().try_for_each(|c| {
                let output = Command::new("cmd")
                    .args(&["/C", c])
                    .output()
                    .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", c, e))?;

                if output.status.success() {
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(anyhow::anyhow!(
                        "Command '{}' failed with error: {}",
                        c,
                        stderr
                    ))
                }
            })
        } else {
            Ok(())
        }
    }
}

impl TweakAction for CommandTweak {
    fn read(&self) -> Result<(), anyhow::Error> {
        if let Some(target_state) = &self.target_state {
            let current_state = self.read_current_state()?;
            if current_state != Some(target_state.clone().into_iter().collect()) {
                return Err(anyhow::anyhow!("Current state does not match target state"));
            }
        }
        Ok(())
    }

    fn apply(&self) -> Result<(), anyhow::Error> {
        self.apply()
    }

    fn revert(&self) -> Result<(), anyhow::Error> {
        self.revert()
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
        let mut updated_tweaks = Vector::new();

        for (index, tweak) in ALL_TWEAKS.iter().cloned().enumerate() {
            let mut tweak = tweak.clone();
            tweak.id = index;
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
                TweakMethod::Command(_) => {
                    // For CommandTweaks, you might set enabled to false by default
                    tweak.enabled = false;
                }
            }
            let updated_tweak = tweak.clone();
            updated_tweaks.push_back(updated_tweak);
        }

        AppState {
            tweak_list: updated_tweaks,
        }
    }
}

struct TweakSwitch {
    child: Switch,
}

impl Widget<Tweak> for TweakSwitch {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Tweak, env: &Env) {
        if let Event::MouseDown(_) = event {
            if data.applying {
                // Do nothing if already applying
                return;
            }

            data.applying = true;
            ctx.request_paint();

            let sink = ctx.get_external_handle();
            let tweak_id = data.id;
            let enabled = data.enabled;
            let data_clone = data.clone();

            std::thread::spawn(move || {
                let success = if !enabled {
                    match data_clone.apply() {
                        Ok(_) => true,
                        Err(e) => {
                            eprintln!("Failed to apply tweak '{}': {}", data_clone.name, e);
                            false
                        }
                    }
                } else {
                    match data_clone.revert() {
                        Ok(_) => false,
                        Err(e) => {
                            eprintln!("Failed to revert tweak '{}': {}", data_clone.name, e);
                            true
                        }
                    }
                };

                sink.submit_command(SET_APPLYING, (tweak_id, false), Target::Auto)
                    .expect("Failed to submit command");

                if success {
                    // Update data.enabled
                    sink.submit_command(UPDATE_TWEAK_ENABLED, (tweak_id, !enabled), Target::Auto)
                        .expect("Failed to submit command");
                }
            });
        }
        self.child.event(ctx, event, &mut data.enabled, env);

        if let Event::MouseDown(_) = event {
            if data.applying {
                // Do nothing if already applying
                return;
            }

            data.applying = true;
            ctx.request_paint();

            let sink = ctx.get_external_handle();
            let tweak_id = data.id;
            let enabled = data.enabled;
            let data_clone = data.clone();

            std::thread::spawn(move || {
                let result = if !enabled {
                    data_clone.apply()
                } else {
                    data_clone.revert()
                };

                let success = result.is_ok();

                if let Err(ref e) = result {
                    println!("Failed to apply/revert tweak '{}': {}", data_clone.name, e);
                } else {
                    println!("Applied/Reverted tweak '{}'", data_clone.name);
                }

                sink.submit_command(SET_APPLYING, (tweak_id, false), Target::Auto)
                    .expect("Failed to submit command");

                if success {
                    // Update data.enabled
                    sink.submit_command(UPDATE_TWEAK_ENABLED, (tweak_id, !enabled), Target::Auto)
                        .expect("Failed to submit command");
                }
            });
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Tweak, env: &Env) {
        self.child.lifecycle(ctx, event, &data.enabled, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Tweak, data: &Tweak, env: &Env) {
        self.child
            .update(ctx, &old_data.enabled, &data.enabled, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Tweak,
        env: &Env,
    ) -> Size {
        self.child.layout(ctx, bc, &data.enabled, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Tweak, env: &Env) {
        self.child.paint(ctx, &data.enabled, env);
    }
}

// Controller to handle apply and revert actions
pub struct TweakController;

impl TweakController {
    pub fn new() -> Self {
        Self
    }
}

impl<W: Widget<Tweak>> druid::widget::Controller<Tweak, W> for TweakController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Tweak,
        env: &Env,
    ) {
        if let Event::Command(cmd) = event {
            if let Some((tweak_id, applying)) = cmd.get(SET_APPLYING) {
                if *tweak_id == data.id {
                    data.applying = *applying;
                    ctx.request_paint();
                }
            } else if let Some((tweak_id, enabled)) = cmd.get(UPDATE_TWEAK_ENABLED) {
                if *tweak_id == data.id {
                    data.enabled = *enabled;
                    ctx.request_paint();
                }
            }
        }
        child.event(ctx, event, data, env);
    }
}

struct ButtonController;

impl<W: Widget<Tweak>> Controller<Tweak, W> for ButtonController {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &Tweak,
        data: &Tweak,
        env: &Env,
    ) {
        if old_data.applying != data.applying {
            ctx.request_paint();
        }
        child.update(ctx, old_data, data, env);
    }

    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Tweak,
        env: &Env,
    ) {
        // Disable the button if applying
        if data.applying {
            return;
        }
        child.event(ctx, event, data, env);
    }
}

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(LocalizedString::new("OC Tool"))
        .window_size((500.0, 400.0));

    let initial_state = AppState::default();

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("launch failed");
}
