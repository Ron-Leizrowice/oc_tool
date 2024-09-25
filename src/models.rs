// src/models.rs

use druid::{im::Vector, Data, Lens};

use crate::{
    actions::TweakAction,
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
    pub method: TweakMethod,
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
            tweak.id = index; // Assign unique ID based on index

            // Initialize 'enabled' based on current system settings
            match &tweak.method {
                TweakMethod::Registry(registry_tweak) => {
                    match registry_tweak.read_current_value() {
                        Ok(current_value) => {
                            // Compare current value with desired value
                            tweak.enabled = current_value == registry_tweak.value;
                        }
                        Err(e) => {
                            tracing::debug!(
                                "Failed to read current value for registry tweak '{}': {}",
                                tweak.name,
                                e
                            );
                            tweak.enabled = false; // Default to disabled on error
                        }
                    }
                }
                TweakMethod::GroupPolicy(gp_tweak) => match gp_tweak.read_current_value() {
                    Ok(GroupPolicyValue::Enabled) => tweak.enabled = true,
                    Ok(GroupPolicyValue::Disabled) => tweak.enabled = false,
                    Err(e) => {
                        tracing::debug!(
                            "Failed to read current value for group policy tweak '{}': {}",
                            tweak.name,
                            e
                        );
                        tweak.enabled = false;
                    }
                },
                TweakMethod::Command(_) => {
                    // Invoke `is_enabled` to determine the current state
                    match tweak.is_enabled() {
                        Ok(_) => {
                            // `is_enabled` updates `tweak.enabled` internally
                        }
                        Err(e) => {
                            tracing::debug!(
                                "Failed to determine enabled state for command tweak '{}': {}",
                                tweak.name,
                                e
                            );
                            tweak.enabled = false; // Default to disabled on error
                        }
                    }
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
