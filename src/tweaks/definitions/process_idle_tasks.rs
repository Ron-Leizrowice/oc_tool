use std::process::Command;

use anyhow::Error;

use crate::tweaks::{TweakId, TweakMethod};

pub struct ProcessIdleTasksTweak {
    pub id: TweakId,
}

impl TweakMethod for ProcessIdleTasksTweak {
    fn initial_state(&self) -> Result<bool, Error> {
        // Since this is an action, it doesn't have a state
        Ok(false)
    }

    fn apply(&self) -> Result<(), Error> {
        tracing::info!("{:?} -> Running Process Idle Tasks.", self.id);

        let mut cmd = Command::new("Rundll32.exe");
        cmd.args(["advapi32.dll,ProcessIdleTasks"]);

        // Run the command
        let output = cmd.output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::msg(format!(
                "{:?} -> Failed to run Process Idle Tasks. Error: {}",
                self.id,
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    fn revert(&self) -> Result<(), Error> {
        Ok(())
    }
}
