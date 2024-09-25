// src/models.rs

use druid::{im::Vector, Data, Lens};

use crate::{
    tweaks::{group_policy_tweaks::GroupPolicyValue, TweakMethod, ALL_TWEAKS},
    ui::widgets::WidgetType,
};

// Base model for any tweak
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

// Application state
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
