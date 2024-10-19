// src/tweaks/msr.rs

use once_cell::sync::Lazy;

use crate::{
    tweaks::{TweakId, TweakMethod},
    utils::winring0::WINRING0_DRIVER,
};

#[derive(Debug, Clone, Copy)]
pub struct CpuInfo {
    pub cores: usize,
    pub _threads: usize,
}

impl CpuInfo {
    pub fn new() -> Self {
        CpuInfo {
            cores: num_cpus::get(),
            _threads: num_cpus::get_physical(),
        }
    }
}

static CPU_INFO: Lazy<CpuInfo> = Lazy::new(CpuInfo::new);

// Ensure MSRTweak implements Clone
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
            .map_err(|_| anyhow::anyhow!("Failed to lock WinRing0"))?;

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

        for step in &self.msrs {
            for core_id in 0..CPU_INFO.cores {
                let current_value = winring0.read_msr(core_id, step.index)?;
                let new_value = if step.state {
                    current_value | (1 << step.bit) // Set bit
                } else {
                    current_value & !(1 << step.bit) // Clear bit
                };
                winring0.write_msr(core_id, step.index, new_value)?;

                // Verify the bit is set or cleared as desired
                let updated_value = winring0.read_msr(core_id, step.index)?;
                let state = ((updated_value >> step.bit) & 1) == if step.state { 1 } else { 0 };
                if !state {
                    tracing::error!(
                        "Failed to set MSR 0x{:X} bit {} on core {}",
                        step.index,
                        step.bit,
                        core_id
                    );
                    return Err(anyhow::anyhow!(
                        "Failed to set MSR 0x{:X} bit {} on core {}",
                        step.index,
                        step.bit,
                        core_id
                    ));
                }
            }

            tracing::info!(
                "Successfully applied MSR 0x{:X} bit {} on all cores.",
                step.index,
                step.bit,
            );
        }

        Ok(())
    }

    /// Reverts all MSR tweaks by resetting the specified bits to their original state.
    fn revert(&self) -> std::result::Result<(), anyhow::Error> {
        // Lock the WinRing0 driver to prevent concurrent access
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock WinRing0"))?;

        for step in &self.msrs {
            for core_id in 0..CPU_INFO.cores {
                let current_value = winring0.read_msr(core_id, step.index)?;
                let new_value = if step.state {
                    current_value & !(1 << step.bit) // Clear bit
                } else {
                    current_value | (1 << step.bit) // Set bit
                };
                winring0.write_msr(core_id, step.index, new_value)?;

                // Verify the bit is reverted as desired
                let updated_value = winring0.read_msr(core_id, step.index)?;
                let state = ((updated_value >> step.bit) & 1) == if step.state { 0 } else { 1 };
                if !state {
                    tracing::error!(
                        "Failed to revert MSR 0x{:X} bit {} on core {}",
                        step.index,
                        step.bit,
                        core_id
                    );
                    return Err(anyhow::anyhow!(
                        "Failed to revert MSR 0x{:X} bit {} on core {}",
                        step.index,
                        step.bit,
                        core_id
                    ));
                }
            }

            tracing::info!(
                "Successfully reverted MSR 0x{:X} bit {} on all cores.",
                step.index,
                step.bit,
            );
        }

        Ok(())
    }
}
