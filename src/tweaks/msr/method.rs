// src/tweaks/msr.rs

use crate::{
    tweaks::{TweakId, TweakMethod},
    utils::{cpu::CPU_INFO, winring0::WINRING0_DRIVER},
};

pub struct MSRTweak {
    pub id: TweakId,
    pub readable: bool,
    pub msrs: Vec<MsrTweakState>,
}

pub struct MsrTweakState {
    pub index: u32,
    pub bit: u32,
    pub state: bool, // whether to set 1 (true) or 0 (false) when enabling the tweak
}

impl MSRTweak {
    /// Creates a set mask by OR-ing all bits that need to be set.
    fn create_set_mask(steps: &[&MsrTweakState]) -> u64 {
        steps
            .iter()
            .filter(|s| s.state)
            .fold(0, |acc, s| acc | (1 << s.bit))
    }

    /// Creates a clear mask by OR-ing all bits that need to be cleared.
    fn create_clear_mask(steps: &[&MsrTweakState]) -> u64 {
        steps
            .iter()
            .filter(|s| !s.state)
            .fold(0, |acc, s| acc | (1 << s.bit))
    }

    /// Applies the set and clear masks to the current value.
    fn apply_masks(current_value: u64, set_mask: u64, clear_mask: u64) -> u64 {
        (current_value | set_mask) & !clear_mask
    }

    /// Reverts the set and clear masks to the expected value.
    fn revert_masks(current_value: u64, set_mask: u64, clear_mask: u64) -> u64 {
        (current_value | set_mask) & !clear_mask
    }
}

impl TweakMethod for MSRTweak {
    /// Retrieves the initial state of all MSR tweaks.
    /// Returns `Ok(true)` if all steps are in the desired state,
    /// `Ok(false)` if any step is not in the desired state,
    /// or an error if the state cannot be determined consistently.
    fn initial_state(&self) -> std::result::Result<bool, anyhow::Error> {
        if !self.readable {
            return Ok(false);
        }

        // Lock the WinRing0 driver to prevent concurrent access
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock WinRing0: {:?}", e))?;

        let mut all_states = Vec::with_capacity(self.msrs.len());

        for step in &self.msrs {
            // Collect state for each step across all cores
            let mut step_states = Vec::with_capacity(CPU_INFO.cores);
            for core_id in 0..CPU_INFO.cores {
                let value = winring0.read_msr(core_id, step.index)?;
                let state = ((value >> step.bit) & 1) == if step.state { 1 } else { 0 };
                step_states.push(state);
            }

            // Verify all cores have the desired state for this step
            if step_states.iter().all(|&s| s) {
                all_states.push(true);
            } else if step_states.iter().all(|&s| !s) {
                all_states.push(false);
            } else {
                // Inconsistent state across cores
                return Err(anyhow::anyhow!(
                    "Inconsistent MSR states for index 0x{:X}, bit {} across cores",
                    step.index,
                    step.bit
                ));
            }
        }

        // Overall initial state is true if all steps are in their desired state
        Ok(all_states.iter().all(|&s| s))
    }

    /// Applies all MSR tweaks by setting or clearing the specified bits.
    fn apply(&self) -> std::result::Result<(), anyhow::Error> {
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock WinRing0"))?;

        // Group MsrTweakState by MSR index
        let mut msr_groups: std::collections::HashMap<u32, Vec<&MsrTweakState>> =
            std::collections::HashMap::new();
        for step in &self.msrs {
            msr_groups.entry(step.index).or_default().push(step);
        }

        for (msr_index, steps) in msr_groups {
            for core_id in 0..CPU_INFO.cores {
                // Read current MSR value
                let current_value = winring0.read_msr(core_id, msr_index)?;

                // Create masks
                let set_mask = MSRTweak::create_set_mask(&steps);
                let clear_mask = MSRTweak::create_clear_mask(&steps);

                // Apply masks
                let new_value = MSRTweak::apply_masks(current_value, set_mask, clear_mask);

                // Write back the modified MSR value
                winring0.write_msr(core_id, msr_index, new_value)?;

                // Verify the changes
                let updated_value = winring0.read_msr(core_id, msr_index)?;
                if updated_value != new_value {
                    tracing::error!(
                        "Failed to apply MSR 0x{:X} on core {}. Expected: 0x{:016X}, Got: 0x{:016X}",
                        msr_index,
                        core_id,
                        new_value,
                        updated_value
                    );
                    return Err(anyhow::anyhow!(
                        "Failed to apply MSR 0x{:X} on core {}",
                        msr_index,
                        core_id
                    ));
                }
            }

            tracing::info!("Successfully applied MSR 0x{:X} on all cores.", msr_index);
        }

        Ok(())
    }

    /// Reverts all MSR tweaks by resetting the specified bits to their original state.
    fn revert(&self) -> std::result::Result<(), anyhow::Error> {
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock WinRing0"))?;

        // Group MsrTweakState by MSR index
        let mut msr_groups: std::collections::HashMap<u32, Vec<&MsrTweakState>> =
            std::collections::HashMap::new();
        for step in &self.msrs {
            msr_groups.entry(step.index).or_default().push(step);
        }

        for (msr_index, steps) in msr_groups {
            for core_id in 0..CPU_INFO.cores {
                // Read current MSR value
                let current_value = winring0.read_msr(core_id, msr_index)?;

                // To revert, invert the set and clear masks
                let set_mask = MSRTweak::create_clear_mask(&steps);
                let clear_mask = MSRTweak::create_set_mask(&steps);

                // Apply masks
                let new_value = MSRTweak::revert_masks(current_value, set_mask, clear_mask);

                // Write back the modified MSR value
                winring0.write_msr(core_id, msr_index, new_value)?;

                // Verify the changes
                let updated_value = winring0.read_msr(core_id, msr_index)?;
                if updated_value != new_value {
                    tracing::error!(
                        "Failed to revert MSR 0x{:X} on core {}. Expected: 0x{:016X}, Got: 0x{:016X}",
                        msr_index,
                        core_id,
                        new_value,
                        updated_value
                    );
                    return Err(anyhow::anyhow!(
                        "Failed to revert MSR 0x{:X} on core {}",
                        msr_index,
                        core_id
                    ));
                }
            }

            tracing::info!("Successfully reverted MSR 0x{:X} on all cores.", msr_index);
        }

        Ok(())
    }
}
