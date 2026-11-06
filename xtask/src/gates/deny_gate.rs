// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-deny";
const FETCH_ATTEMPTS: usize = 3;

// cargo-deny's exit code carries one bit per check that failed.
const CHECKS: [(i32, &str); 4] = [
    (1, "advisories"),
    (2, "bans"),
    (4, "licenses"),
    (8, "sources"),
];

// What the dependencies are, rather than whether they are used: known
// advisories, banned or duplicated crates, licences outside deny.toml's
// allow-list, and anything not from crates.io. It runs last because its
// advisories come from the RustSec database, not from the change: a newly
// published advisory turns it red on an untouched checkout, and that must not
// hide what the house-rule gates have to say about the change itself.
//
// The database is fetched first and on its own, up to three times, and the
// check then runs --frozen: locked and offline. A fetch that keeps failing is
// reported as a gate that could not run, never as an advisory found. A
// deny.toml it cannot read still exits 1, as advisories does; the streamed
// output says which.
pub struct DenyGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
}

impl<'a> DenyGate<'a> {
    pub fn new(runner: &'a dyn CommandRunner, manifest_path: String) -> Self {
        Self {
            runner,
            manifest_path,
        }
    }

    fn args(&self, tail: &[&str]) -> Vec<String> {
        let mut args = vec![
            String::from("deny"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        args.extend(tail.iter().map(|arg| String::from(*arg)));
        args
    }

    fn fetched(&self) -> Result<(), GateFailure> {
        let mut exit_code = None;
        for _ in 0..FETCH_ATTEMPTS {
            let outcome = self
                .runner
                .run_capturing("cargo", &self.args(&["fetch", "db"]))?;
            if outcome.is_success() {
                return Ok(());
            }
            exit_code = outcome.exit_code();
        }
        Err(GateFailure::CouldNotRun { exit_code })
    }

    fn rejected(code: i32) -> Option<String> {
        let all = CHECKS.iter().fold(0, |bits, (bit, _)| bits | bit);
        if code & !all != 0 {
            return None;
        }
        let names: Vec<&str> = CHECKS
            .iter()
            .filter(|(bit, _)| code & bit != 0)
            .map(|(_, name)| *name)
            .collect();
        Some(names.join(", "))
    }
}

impl Gate for DenyGate<'_> {
    fn label(&self) -> String {
        String::from("Supply chain")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled {
                binary: String::from(BINARY),
            });
        }

        self.fetched()?;

        // --frozen: the lock file is read and never written, and nothing is
        // fetched -- the database came in above.
        let args = self.args(&["--frozen", "check"]);
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            Some(code) => Err(Self::rejected(code).map_or(
                GateFailure::CouldNotRun {
                    exit_code: Some(code),
                },
                |checks| GateFailure::Rejected { checks },
            )),
            None => Err(GateFailure::CouldNotRun { exit_code: None }),
        }
    }
}
