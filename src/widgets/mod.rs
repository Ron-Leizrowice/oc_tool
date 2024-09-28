// src/widgets/mod.rs

pub mod button;
pub mod switch;
use std::{collections::HashMap, sync::LazyLock};

use crate::tweaks::TweakId;

/// Enum representing the different widget types for a tweak.
#[derive(Clone, Debug)]
pub enum TweakWidget {
    Switch,
    Button,
}
pub static TWEAK_WIDGETS: LazyLock<HashMap<TweakId, TweakWidget>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(TweakId::LargeSystemCache, TweakWidget::Switch);
    map.insert(TweakId::SystemResponsiveness, TweakWidget::Switch);
    map.insert(TweakId::DisableHWAcceleration, TweakWidget::Switch);
    map.insert(TweakId::Win32PrioritySeparation, TweakWidget::Switch);
    map.insert(TweakId::DisableCoreParking, TweakWidget::Switch);
    map.insert(TweakId::SeLockMemoryPrivilege, TweakWidget::Switch);
    map.insert(TweakId::UltimatePerformancePlan, TweakWidget::Switch);
    map
});
