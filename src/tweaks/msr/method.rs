// src/tweaks/msr.rs

use anyhow::{Context, Result};
use indexmap::IndexMap;
use tracing::{debug, error};

use crate::{
    tweaks::{TweakId, TweakMethod, TweakOption},
    utils::{cpu::CPU_INFO, winring0::WINRING0_DRIVER},
};

/// Represents a single MSR modification, including the MSR index, bit position,
/// and the desired state (set to 1 or 0).
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct MsrState {
    /// MSR index (e.g., 0x10).
    pub index: u32,
    /// Bit position within the MSR to modify.
    pub bit: u32,
    /// Desired state: `true` to set the bit to 1, `false` to clear it to 0.
    pub state: bool,
}

/// Defines a set of MSR modifications, which in combination
/// make up a single tweak.
#[derive(Debug)]
pub struct MSRTweak {
    /// Unique ID for the tweak.
    pub id: TweakId,
    /// Indicates whether the tweak is readable.
    pub readable: bool,
    /// Mapping from `TweakOption` to a list of MSR modifications.
    pub options: IndexMap<TweakOption, Vec<MsrState>>,
}

impl MSRTweak {
    /// Creates a set mask by OR-ing all bits that need to be set.
    fn create_set_mask(steps: &[&MsrState]) -> u64 {
        steps
            .iter()
            .filter(|s| s.state)
            .fold(0, |acc, s| acc | (1u64 << s.bit))
    }

    /// Creates a clear mask by OR-ing all bits that need to be cleared.
    fn create_clear_mask(steps: &[&MsrState]) -> u64 {
        steps
            .iter()
            .filter(|s| !s.state)
            .fold(0, |acc, s| acc | (1u64 << s.bit))
    }

    /// Applies the set and clear masks to the current MSR value.
    fn apply_masks(current_value: u64, set_mask: u64, clear_mask: u64) -> u64 {
        (current_value | set_mask) & !clear_mask
    }

    /// Reverts the set and clear masks to the expected value.
    fn revert_masks(current_value: u64, set_mask: u64, clear_mask: u64) -> u64 {
        (current_value | set_mask) & !clear_mask
    }
}

impl TweakMethod for MSRTweak {
    /// Checks the current state of the MSR tweak and returns the corresponding `TweakOption`.
    ///
    /// # Returns
    /// - `Ok(TweakOption)` indicating the current state.
    /// - `Err(anyhow::Error)` if the operation fails.
    fn initial_state(&self) -> Result<TweakOption> {
        debug!(
            "{:?} -> Determining the initial state of the MSR tweak.",
            self.id
        );

        if !self.readable {
            return Ok(TweakOption::Enabled(false));
        }

        // Lock the WinRing0 driver to prevent concurrent access
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock WinRing0: {:?}", e))?;

        // Iterate through all possible options to find which one matches the current state
        for (option, modifications) in &self.options {
            let mut all_match = true;

            for step in modifications {
                for core_id in 0..CPU_INFO.cores {
                    let current_value =
                        winring0.read_msr(core_id, step.index).with_context(|| {
                            format!(
                                "{:?} -> Failed to read MSR 0x{:X} on core {}",
                                self.id, step.index, core_id
                            )
                        })?;

                    let bit_state = ((current_value >> step.bit) & 1) == 1;
                    if bit_state != step.state {
                        all_match = false;
                        break;
                    }
                }

                if !all_match {
                    break;
                }
            }

            if all_match {
                tracing::debug!("{:?} -> Current state matches {:?}.", self.id, option);
                return Ok(option.clone());
            }
        }

        // If no matching option is found, consider it as Default
        tracing::debug!(
            "{:?} -> Current state does not match any custom options. Reverting to Default.",
            self.id
        );
        Ok(TweakOption::Enabled(false))
    }

    /// Applies the MSR tweak based on the selected `TweakOption`.
    ///
    /// # Parameters
    ///
    /// - `option`: The `TweakOption` to apply (Default or Custom).
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations succeed.
    /// - `Err(anyhow::Error)` if any operation fails, after attempting rollback.
    fn apply(&self, option: TweakOption) -> Result<()> {
        debug!(
            "Applying MSR tweak '{:?}' with option: {:?}.",
            self.id, option
        );

        let modifications = self.options.get(&option).context(format!(
            "{:?} -> No MSR modifications found for option: {:?}",
            self.id, option
        ))?;

        // Lock the WinRing0 driver to prevent concurrent access
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock WinRing0: {:?}", e))?;

        // Wrap the apply logic in a closure to handle errors and perform rollback
        let result: Result<(), anyhow::Error> = (|| -> Result<(), anyhow::Error> {
            for step in modifications {
                for core_id in 0..CPU_INFO.cores {
                    // Read current MSR value
                    let current_value =
                        winring0.read_msr(core_id, step.index).with_context(|| {
                            format!(
                                "{:?} -> Failed to read MSR 0x{:X} on core {}",
                                self.id, step.index, core_id
                            )
                        })?;

                    // Create masks
                    let set_mask = MSRTweak::create_set_mask(&[step]);
                    let clear_mask = MSRTweak::create_clear_mask(&[step]);

                    // Apply masks
                    let new_value = MSRTweak::apply_masks(current_value, set_mask, clear_mask);

                    // Write back the modified MSR value
                    winring0
                        .write_msr(core_id, step.index, new_value)
                        .with_context(|| {
                            format!(
                                "{:?} -> Failed to write MSR 0x{:X} on core {}",
                                self.id, step.index, core_id
                            )
                        })?;

                    // Verify the changes
                    let updated_value =
                        winring0.read_msr(core_id, step.index).with_context(|| {
                            format!(
                                "{:?} -> Failed to read MSR 0x{:X} on core {} after write",
                                self.id, step.index, core_id
                            )
                        })?;
                    if updated_value != new_value {
                        tracing::error!(
                            "{:?} -> Failed to apply MSR 0x{:X} on core {}. Expected: 0x{:016X}, Got: 0x{:016X}",
                            self.id,
                            step.index,
                            core_id,
                            new_value,
                            updated_value
                        );
                        return Err(anyhow::anyhow!(
                            "Failed to apply MSR 0x{:X} on core {}",
                            step.index,
                            core_id
                        ));
                    }
                }

                tracing::info!(
                    "{:?} -> Successfully applied MSR 0x{:X} on all cores.",
                    self.id,
                    step.index
                );
            }

            Ok(())
        })();

        if let Err(e) = result {
            // An error occurred during apply
            error!(
                "{:?} -> Error occurred during apply: {}. Attempting rollback.",
                self.id, e
            );

            return Err(e);
        }

        debug!(
            "{:?} -> Successfully applied MSR tweak with option: {:?}.",
            self.id, option
        );
        Ok(())
    }

    /// Reverts the MSR tweak by restoring the `Default` option.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations succeed.
    /// - `Err(anyhow::Error)` if any operation fails, after attempting rollback.
    fn revert(&self) -> Result<()> {
        debug!("{:?} -> Reverting MSR tweak to Default.", self.id);

        let default_modifications =
            self.options
                .get(&TweakOption::Enabled(false))
                .context(format!(
                    "{:?} -> No MSR modifications found for Default option.",
                    self.id
                ))?;

        // Lock the WinRing0 driver to prevent concurrent access
        let winring0 = WINRING0_DRIVER
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock WinRing0: {:?}", e))?;

        // Wrap the revert logic in a closure to handle errors and perform rollback
        let result: Result<(), anyhow::Error> = (|| -> Result<(), anyhow::Error> {
            for step in default_modifications {
                for core_id in 0..CPU_INFO.cores {
                    // Read current MSR value
                    let current_value =
                        winring0.read_msr(core_id, step.index).with_context(|| {
                            format!(
                                "{:?} -> Failed to read MSR 0x{:X} on core {}",
                                self.id, step.index, core_id
                            )
                        })?;

                    // Create masks for reverting
                    let set_mask = MSRTweak::create_clear_mask(&[step]);
                    let clear_mask = MSRTweak::create_set_mask(&[step]);

                    // Apply masks to revert
                    let new_value = MSRTweak::revert_masks(current_value, set_mask, clear_mask);

                    // Write back the reverted MSR value
                    winring0
                        .write_msr(core_id, step.index, new_value)
                        .with_context(|| {
                            format!(
                                "{:?} -> Failed to write MSR 0x{:X} on core {}",
                                self.id, step.index, core_id
                            )
                        })?;

                    // Verify the revert
                    let updated_value =
                        winring0.read_msr(core_id, step.index).with_context(|| {
                            format!(
                                "{:?} -> Failed to read MSR 0x{:X} on core {} after revert",
                                self.id, step.index, core_id
                            )
                        })?;
                    if updated_value != new_value {
                        tracing::error!(
                            "{:?} -> Failed to revert MSR 0x{:X} on core {}. Expected: 0x{:016X}, Got: 0x{:016X}",
                            self.id,
                            step.index,
                            core_id,
                            new_value,
                            updated_value
                        );
                        return Err(anyhow::anyhow!(
                            "Failed to revert MSR 0x{:X} on core {}",
                            step.index,
                            core_id
                        ));
                    }
                }

                tracing::info!(
                    "{:?} -> Successfully reverted MSR 0x{:X} on all cores.",
                    self.id,
                    step.index
                );
            }

            Ok(())
        })();

        if let Err(e) = result {
            // An error occurred during revert
            error!(
                "{:?} -> Error occurred during revert: {}. Attempting rollback.",
                self.id, e
            );

            return Err(e);
        }

        debug!(
            "{:?} -> Successfully reverted MSR tweak to Default.",
            self.id
        );
        Ok(())
    }
}
