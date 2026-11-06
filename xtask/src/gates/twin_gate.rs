// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-twin4rust";

pub struct TwinGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
    packages: Vec<String>,
}

impl<'a> TwinGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        manifest_path: String,
        packages: Vec<String>,
    ) -> Self {
        Self {
            runner,
            manifest_path,
            packages,
        }
    }
}

impl Gate for TwinGate<'_> {
    fn label(&self) -> String {
        String::from("Mirrored tests")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled {
                binary: String::from(BINARY),
            });
        }

        let mut args = vec![
            String::from("twin4rust"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }

        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            _ => Err(GateFailure::Unmirrored),
        }
    }
}
