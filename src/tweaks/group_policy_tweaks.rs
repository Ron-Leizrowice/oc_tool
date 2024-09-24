// src/tweaks/group_policy_tweaks.rs

use once_cell::sync::Lazy;

use crate::{GroupPolicyTweak, Tweak, TweakMethod};

pub static SE_LOCK_MEMORY_PRIVILEGE: Lazy<Tweak> = Lazy::new(|| Tweak {
    id: 0,
    name: "SeLockMemoryPrivilege".to_string(),
    enabled: false,
    description: "Assigns the 'Lock pages in memory' privilege to the current user.".to_string(),
    config: TweakMethod::GroupPolicy(GroupPolicyTweak {
        key: "SeLockMemoryPrivilege".to_string(),
        value: None,
    }),
    requires_restart: true,
    applying: false,
});
