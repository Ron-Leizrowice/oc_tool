// src/tweaks/registry_tweaks.rs

use once_cell::sync::Lazy;

use crate::{RegistryKeyValue, RegistryTweak, Tweak, TweakMethod, WidgetType};

pub static LARGE_SYSTEM_CACHE: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
    id: 0,
    name: "LargeSystemCache".to_string(),
    description: "Optimizes system memory management by adjusting the LargeSystemCache setting.".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management".to_string(),
        name: "LargeSystemCache".to_string(),
        value: RegistryKeyValue::Dword(1),
        default_value: RegistryKeyValue::Dword(0),
    }),
    requires_restart: false,
    applying: false,
}
});

pub static SYSTEM_RESPONSIVENESS: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
    id: 0,
    name: "SystemResponsiveness".to_string(),
    description: "Optimizes system responsiveness by adjusting the SystemResponsiveness setting.".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile".to_string(),
        name: "SystemResponsiveness".to_string(),
        value: RegistryKeyValue::Dword(0),
        default_value: RegistryKeyValue::Dword(20),
    }),
    requires_restart: false,
    applying: false,
}
});

pub static DISABLE_HW_ACCELERATION: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "DisableHWAcceleration".to_string(),
    description: "Disables hardware acceleration for the current user.".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Avalon.Graphics".to_string(),
        name: "DisableHWAcceleration".to_string(),
        value: RegistryKeyValue::Dword(1),
        default_value: RegistryKeyValue::Dword(0),
    }),
    requires_restart: false,
    applying: false,
});

pub static WIN_32_PRIORITY_SEPARATION: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "Win32PrioritySeparation".to_string(),
    description:
        "Optimizes system responsiveness by adjusting the Win32PrioritySeparation setting."
            .to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\PriorityControl".to_string(),
        name: "Win32PrioritySeparation".to_string(),
        value: RegistryKeyValue::Dword(26),
        default_value: RegistryKeyValue::Dword(2),
    }),
    requires_restart: false,
    applying: false,
});

pub static DISABLE_LOW_DISK_CHECK: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "DisableLowDiskCheck".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    description: "Disables the low disk space check for the current user.".to_string(),
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer"
            .to_string(),
        name: "NoLowDiskSpaceChecks".to_string(),
        value: RegistryKeyValue::Dword(1),
        default_value: RegistryKeyValue::Dword(0),
    }),
    requires_restart: false,
    applying: false,
});

pub static DISABLE_CORE_PARKING: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
    id: 0,
    name: "DisableCoreParking".to_string(),
    widget: WidgetType::Switch,
    enabled: false,
    description: "Disables core parking to improve system performance.".to_string(),
    config: TweakMethod::Registry(RegistryTweak {
        key: "HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Control\\Power\\PowerSettings\\54533251-82be-4824-96c1-47b60b740d00\\0cc5b647-c1df-4637-891a-dec35c318583".to_string(),
        name: "ValueMax".to_string(),
        value: RegistryKeyValue::Dword(0),
        default_value: RegistryKeyValue::Dword(64),
    }),
    requires_restart: true,
    applying: false,
}
});
