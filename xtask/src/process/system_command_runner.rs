// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::process::command_outcome::CommandOutcome;
use crate::process::command_runner::CommandRunner;
use crate::process::launch_failure::LaunchFailure;
use std::env::consts::EXE_SUFFIX;
use std::env::split_paths;
use std::env::var_os;
use std::path::PathBuf;
use std::process::Command;

pub struct SystemCommandRunner;

impl SystemCommandRunner {
    pub fn new() -> Self {
        Self
    }

    fn locate(binary_name: &str) -> Option<PathBuf> {
        let exe_name = format!("{binary_name}{EXE_SUFFIX}");
        let path_var = var_os("PATH")?;
        split_paths(&path_var)
            .map(|directory| directory.join(&exe_name))
            .find(|candidate| candidate.is_file())
    }
}

impl Default for SystemCommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRunner for SystemCommandRunner {
    fn is_available(&self, binary_name: &str) -> bool {
        Self::locate(binary_name).is_some()
    }

    fn run_streaming(&self, program: &str, args: &[String]) -> Result<Option<i32>, LaunchFailure> {
        Command::new(program)
            .args(args)
            .status()
            .map(|status| status.code())
            .map_err(|error| LaunchFailure::new(program, &error.to_string()))
    }

    fn run_capturing(
        &self,
        program: &str,
        args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|error| LaunchFailure::new(program, &error.to_string()))?;

        Ok(CommandOutcome::new(
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}
