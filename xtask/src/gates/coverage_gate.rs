// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-llvm-cov";

// No line, region or function of beggar left uncovered: over every tier, and
// over one tier alone -- the conformance tier, so that the crate holds nothing
// the specification's vectors do not ask for
// (P1-ADR-NoLineTheSpecificationDoesNotAskFor). One tier alone is its tests,
// filtered by the module they live in.
pub struct CoverageGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
    packages: Vec<String>,
    tier: Option<String>,
}

impl<'a> CoverageGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        manifest_path: String,
        packages: Vec<String>,
        tier: Option<String>,
    ) -> Self {
        Self {
            runner,
            manifest_path,
            packages,
            tier,
        }
    }

    fn scope(&self) -> String {
        match &self.tier {
            Some(tier) => format!("the {tier} tier alone"),
            None => String::from("every tier"),
        }
    }
}

impl Gate for CoverageGate<'_> {
    fn label(&self) -> String {
        format!("Coverage, {}", self.scope())
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled {
                binary: String::from(BINARY),
            });
        }
        let mut args = vec![
            String::from("llvm-cov"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }
        args.push(String::from("--summary-only"));
        for check in [
            "--fail-uncovered-lines",
            "--fail-uncovered-regions",
            "--fail-uncovered-functions",
        ] {
            args.push(String::from(check));
            args.push(String::from("0"));
        }
        if let Some(tier) = &self.tier {
            args.push(String::from("--"));
            args.push(format!("{tier}::"));
        }
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            _ => Err(GateFailure::Uncovered {
                scope: self.scope(),
            }),
        }
    }
}
