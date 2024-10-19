// src/tweaks/group_policy/mod.rs
pub mod method;

use method::{GroupPolicyTweak, GroupPolicyValue};

use super::{Tweak, TweakCategory};
use crate::tweaks::TweakId;

pub fn all_group_policy_tweaks<'a>() -> Vec<(TweakId, Tweak<'a>)> {
    vec![(TweakId::SeLockMemoryPrivilege, se_lock_memory_privilege())]
}

pub fn se_lock_memory_privilege<'a>() -> Tweak<'a> {
    Tweak::group_policy_tweak(
        "SeLockMemoryPrivilege",
        "The SeLockMemoryPrivilege group policy setting allows a process to lock pages in physical memory, preventing them from being paged out to disk. This can improve performance for applications that require fast, consistent access to critical data by keeping it always available in RAM.",
        TweakCategory::Memory,
        GroupPolicyTweak {
            id: TweakId::SeLockMemoryPrivilege,
            key: "SeLockMemoryPrivilege",
            value: GroupPolicyValue::Enabled,
        },
        true,
    )
}
