// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl CommandOutcome {
    pub fn new(exit_code: Option<i32>, stdout: String, stderr: String) -> Self {
        Self {
            exit_code,
            stdout,
            stderr,
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    // None means the process was killed by a signal rather than exiting, which
    // is a failure and not an unknown.
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0)
    }

    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    pub fn stdout(&self) -> &str {
        &self.stdout
    }
}
