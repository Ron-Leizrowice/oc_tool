// src/tweaks/registry_tweaks.rs

use once_cell::sync::Lazy;

use crate::{RegistryKeyValue, RegistryTweak, Tweak, TweakMethod};

pub static LARGE_SYSTEM_CACHE: Lazy<Tweak> = Lazy::new(|| {
    Tweak {
    id: 0,
    name: "LargeSystemCache".to_string(),
    enabled: false,
    description: "Optimizes system memory management by adjusting the LargeSystemCache setting.".to_string(),
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
