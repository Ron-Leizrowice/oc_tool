// src/tweaks/mod.rs

use command_tweaks::{CommandTweak, ENABLE_ULTIMATE_PERFORMANCE_PLAN, PROCESS_IDLE_TASKS};
use druid::{im::Vector, Data};
use group_policy_tweaks::{GroupPolicyTweak, SE_LOCK_MEMORY_PRIVILEGE};
use once_cell::sync::Lazy;
use registry_tweaks::{
    RegistryTweak, DISABLE_CORE_PARKING, DISABLE_HW_ACCELERATION, DISABLE_LOW_DISK_CHECK,
    LARGE_SYSTEM_CACHE, SYSTEM_RESPONSIVENESS, WIN_32_PRIORITY_SEPARATION,
};

use crate::models::Tweak;

pub mod command_tweaks;
pub mod group_policy_tweaks;
pub mod registry_tweaks;

#[derive(Debug, Clone, Data)]
pub enum TweakMethod {
    Registry(RegistryTweak),
    GroupPolicy(GroupPolicyTweak),
    Command(CommandTweak),
}

pub static ALL_TWEAKS: Lazy<Vector<&Tweak>> = Lazy::new(|| {
    Vector::from(vec![
        &*LARGE_SYSTEM_CACHE,
        &*SYSTEM_RESPONSIVENESS,
        &*DISABLE_HW_ACCELERATION,
        &*WIN_32_PRIORITY_SEPARATION,
        &*DISABLE_LOW_DISK_CHECK,
        &*SE_LOCK_MEMORY_PRIVILEGE,
        &*PROCESS_IDLE_TASKS,
        &*DISABLE_CORE_PARKING,
        &*ENABLE_ULTIMATE_PERFORMANCE_PLAN,
    ])
});
