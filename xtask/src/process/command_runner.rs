// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::process::command_outcome::CommandOutcome;
use crate::process::launch_failure::LaunchFailure;

pub trait CommandRunner {
    fn is_available(&self, binary_name: &str) -> bool;

    fn run_streaming(&self, program: &str, args: &[String]) -> Result<Option<i32>, LaunchFailure>;

    fn run_capturing(
        &self,
        program: &str,
        args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure>;
}
