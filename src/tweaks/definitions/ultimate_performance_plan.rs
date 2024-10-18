// src/tweaks/definitions/ultimate_performance_plan.rs

use windows::core::GUID;

use super::TweakId;
use crate::{
    power::{
        duplicate_power_scheme, get_active_power_scheme, get_all_power_schemes,
        set_active_power_scheme, PowerScheme,
    },
    tweaks::TweakMethod,
};

const ULTIMATE_PERFORMANCE_POWER_SCHEME_GUID: GUID =
    GUID::from_u128(0xe9a42b02_d5df_448d_aa00_03f14749eb61);

pub struct UltimatePerformancePlan {
    id: TweakId,
    default_power_plan: PowerScheme,
}

impl UltimatePerformancePlan {
    pub fn new() -> Self {
        Self {
            id: TweakId::UltimatePerformancePlan,
            default_power_plan: get_active_power_scheme().unwrap(),
        }
    }
}

impl TweakMethod for UltimatePerformancePlan {
    fn initial_state(&self) -> Result<bool, anyhow::Error> {
        tracing::debug!("{:?}-> Checking initial state", self.id);
        Ok(self.default_power_plan.guid == ULTIMATE_PERFORMANCE_POWER_SCHEME_GUID)
    }

    fn apply(&self) -> Result<(), anyhow::Error> {
        let available_schemes = get_all_power_schemes().expect("Failed to list power schemes");
        // check if any are called "Ultimate Performance"
        match available_schemes
            .iter()
            .find(|scheme| scheme.guid == ULTIMATE_PERFORMANCE_POWER_SCHEME_GUID)
        {
            Some(scheme) => {
                set_active_power_scheme(&scheme.guid)
                    .expect("Failed to set ultimate performance power scheme");
            }
            None => {
                // create the ultimate performance power plan
                duplicate_power_scheme(&ULTIMATE_PERFORMANCE_POWER_SCHEME_GUID)
                    .expect("Failed to duplicate ultimate performance power plan");
                let available_schemes =
                    get_all_power_schemes().expect("Failed to get power schemes");
                match available_schemes
                    .iter()
                    .find(|scheme| scheme.name == "Ultimate Performance")
                {
                    Some(scheme) => {
                        set_active_power_scheme(&scheme.guid).expect("Failed to set power scheme");
                    }
                    None => {
                        return Err(anyhow::anyhow!(
                            "Failed to create and apply Ultimate Performance power plan"
                        ));
                    }
                }
            }
        }

        tracing::debug!("{:?}-> Applied Ultimate Performance power plan", self.id);
        Ok(())
    }

    fn revert(&self) -> Result<(), anyhow::Error> {
        set_active_power_scheme(&self.default_power_plan.guid)?;
        tracing::debug!("{:?}-> Reverted Ultimate Performance power plan", self.id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let tweak = UltimatePerformancePlan::new();
        assert!(!tweak.initial_state().unwrap());
    }

    #[test]
    fn test_apply() {
        let tweak = UltimatePerformancePlan::new();

        match tweak.apply() {
            Ok(_) => {
                let active_scheme = get_active_power_scheme();
                assert_eq!(active_scheme.unwrap().name, "Ultimate Performance");
            }
            Err(e) => {
                panic!("Failed to apply tweak: {:?}", e);
            }
        }
    }

    #[test]
    fn test_revert() {
        let tweak = UltimatePerformancePlan::new();
        tweak.apply().unwrap();
        tweak.revert().unwrap();
        assert!(!tweak.initial_state().unwrap());
    }
}
