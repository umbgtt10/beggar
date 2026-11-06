// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use xtask::process::command_outcome::CommandOutcome;
use xtask::process::command_runner::CommandRunner;
use xtask::process::launch_failure::LaunchFailure;

// The seam every gate is constructed against, faked once so the gate tests
// share one fake rather than each growing their own.
pub struct FakeCommandRunner {
    available: bool,
    streaming_code: Option<i32>,
    capturing_code: Option<i32>,
    stdout: String,
    calls: RefCell<Vec<Vec<String>>>,
    programs: RefCell<Vec<String>>,
}

impl FakeCommandRunner {
    pub fn new() -> Self {
        Self {
            available: true,
            streaming_code: Some(0),
            capturing_code: Some(0),
            stdout: String::new(),
            calls: RefCell::new(Vec::new()),
            programs: RefCell::new(Vec::new()),
        }
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    // A captured run answers with the streaming code too, unless told otherwise:
    // a gate that captures one command and streams another needs the two apart.
    pub fn with_capturing_code(mut self, capturing_code: Option<i32>) -> Self {
        self.capturing_code = capturing_code;
        self
    }

    pub fn with_streaming_code(mut self, streaming_code: Option<i32>) -> Self {
        self.streaming_code = streaming_code;
        self.capturing_code = streaming_code;
        self
    }

    pub fn with_stdout(mut self, stdout: &str) -> Self {
        self.stdout = String::from(stdout);
        self
    }

    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }

    // Recorded separately from the arguments, because a gate that started
    // invoking the wrong program with the right arguments would otherwise look
    // identical to one that worked.
    pub fn programs(&self) -> Vec<String> {
        self.programs.borrow().clone()
    }
}

impl Default for FakeCommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn is_available(&self, _binary_name: &str) -> bool {
        self.available
    }

    fn run_streaming(&self, program: &str, args: &[String]) -> Result<Option<i32>, LaunchFailure> {
        self.programs.borrow_mut().push(String::from(program));
        self.calls.borrow_mut().push(args.to_vec());
        Ok(self.streaming_code)
    }

    fn run_capturing(
        &self,
        program: &str,
        args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure> {
        self.programs.borrow_mut().push(String::from(program));
        self.calls.borrow_mut().push(args.to_vec());
        Ok(CommandOutcome::new(
            self.capturing_code,
            self.stdout.clone(),
            String::new(),
        ))
    }
}
