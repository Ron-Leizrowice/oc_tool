// src/tweaks/group_policy_tweaks.rs

use once_cell::sync::Lazy;

use crate::{GroupPolicyTweak, GroupPolicyValue, Tweak, TweakMethod, WidgetType};

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
