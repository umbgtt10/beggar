// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-iceberg4rust";
const OFFENDERS_FOUND: i32 = 2;

pub struct IcebergGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
    packages: Vec<String>,
    threshold: String,
}

impl<'a> IcebergGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        manifest_path: String,
        packages: Vec<String>,
        threshold: String,
    ) -> Self {
        Self {
            runner,
            manifest_path,
            packages,
            threshold,
        }
    }
}

impl Gate for IcebergGate<'_> {
    fn label(&self) -> String {
        String::from("File risk")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled {
                binary: String::from(BINARY),
            });
        }

        // The ceiling travels as a string so it reaches the CLI unchanged. Formatting
        // a float would render it with the current locale's separator: a ceiling of
        // 9.5 arrives as `9,5` where the comma is the decimal mark, and that does not
        // parse. This repository's ceiling is a whole number today, so nothing would
        // go wrong right now -- the string is what keeps that true after the first
        // edit to a fractional one.
        let mut args = vec![
            String::from("iceberg4rust"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
            String::from("--threshold"),
            self.threshold.clone(),
        ];
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }

        // 2 is the tool's own "offenders found"; anything else non-zero means it
        // could not run at all.
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            Some(OFFENDERS_FOUND) => Err(GateFailure::AboveCeiling {
                ceiling: self.threshold.clone(),
            }),
            code => Err(GateFailure::CouldNotRun { exit_code: code }),
        }
    }
}
