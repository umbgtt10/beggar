// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::fmt;

// A program that could not be started at all -- missing, not executable,
// refused -- with the reason the system gave. A program that started and then
// failed is not this: its exit code is the gate's to judge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchFailure {
    program: String,
    reason: String,
}

impl LaunchFailure {
    pub fn new(program: &str, reason: &str) -> Self {
        Self {
            program: String::from(program),
            reason: String::from(reason),
        }
    }
}

impl fmt::Display for LaunchFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "failed to launch {}: {}",
            self.program, self.reason
        )
    }
}
