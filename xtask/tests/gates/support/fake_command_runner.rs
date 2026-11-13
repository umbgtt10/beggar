// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::collections::VecDeque;
use xtask::process::command_outcome::CommandOutcome;
use xtask::process::command_runner::CommandRunner;
use xtask::process::launch_failure::LaunchFailure;

// The seam every gate is constructed against, faked once so the gate tests
// share one fake rather than each growing their own.
pub struct FakeCommandRunner {
    available: bool,
    launches: bool,
    streaming_code: Option<i32>,
    capturing_code: Option<i32>,
    capturing_codes: RefCell<VecDeque<Option<i32>>>,
    stdout: String,
    replies: RefCell<VecDeque<String>>,
    calls: RefCell<Vec<Vec<String>>>,
    programs: RefCell<Vec<String>>,
}

impl FakeCommandRunner {
    pub fn new() -> Self {
        Self {
            available: true,
            launches: true,
            streaming_code: Some(0),
            capturing_code: Some(0),
            capturing_codes: RefCell::new(VecDeque::new()),
            stdout: String::new(),
            replies: RefCell::new(VecDeque::new()),
            calls: RefCell::new(Vec::new()),
            programs: RefCell::new(Vec::new()),
        }
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    // A program that is installed but cannot be started: every run fails to
    // launch.
    pub fn with_launch_failure(mut self) -> Self {
        self.launches = false;
        self
    }

    // A captured run answers with the streaming code too, unless told otherwise:
    // a gate that captures one command and streams another needs the two apart.
    pub fn with_capturing_code(mut self, capturing_code: Option<i32>) -> Self {
        self.capturing_code = capturing_code;
        self
    }

    // One exit code per capturing call, in order, for a caller whose questions
    // fare differently; once they run out, every call gets
    // `with_capturing_code`'s answer.
    pub fn with_capturing_codes(self, codes: &[Option<i32>]) -> Self {
        *self.capturing_codes.borrow_mut() = codes.iter().copied().collect();
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

    // One stdout per capturing call, in order, for a caller that asks more than
    // one question; once they run out, every call gets `with_stdout`'s answer.
    pub fn with_replies(self, replies: &[&str]) -> Self {
        *self.replies.borrow_mut() = replies.iter().map(|reply| String::from(*reply)).collect();
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

    fn record(&self, program: &str, args: &[String]) -> Result<(), LaunchFailure> {
        self.programs.borrow_mut().push(String::from(program));
        self.calls.borrow_mut().push(args.to_vec());
        if self.launches {
            Ok(())
        } else {
            Err(LaunchFailure::new(program, "refused by the fake"))
        }
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
        self.record(program, args)?;
        Ok(self.streaming_code)
    }

    fn run_capturing(
        &self,
        program: &str,
        args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure> {
        self.record(program, args)?;
        let stdout = self
            .replies
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| self.stdout.clone());
        let code = self
            .capturing_codes
            .borrow_mut()
            .pop_front()
            .unwrap_or(self.capturing_code);
        Ok(CommandOutcome::new(code, stdout, String::new()))
    }
}
