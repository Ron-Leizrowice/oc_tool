// src/constants.rs

use druid::Selector;
use once_cell::sync::Lazy;

pub static USERNAME: Lazy<String> = Lazy::new(whoami::username);

pub static POLICY_CREATE_ACCOUNT: u32 = 0x00000010;
pub static POLICY_LOOKUP_NAMES: u32 = 0x00000800;

pub static SET_APPLYING: Selector<(usize, bool)> = Selector::new("my_app.set_applying");
pub static UPDATE_TWEAK_ENABLED: Selector<(usize, bool)> =
    Selector::new("my_app.update_tweak_enabled");
