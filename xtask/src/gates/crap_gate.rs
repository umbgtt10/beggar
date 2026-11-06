// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::crap::crap_report_parser::CrapReportParser;
use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

const BINARY: &str = "cargo-crap4rust";
const STDOUT_TAIL_LINES: usize = 30;

pub struct CrapGate<'a> {
    runner: &'a dyn CommandRunner,
    parser: &'a CrapReportParser,
    manifest_path: String,
    packages: Vec<String>,
    threshold: String,
}

impl<'a> CrapGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        parser: &'a CrapReportParser,
        manifest_path: String,
        packages: Vec<String>,
        threshold: String,
    ) -> Self {
        Self {
            runner,
            parser,
            manifest_path,
            packages,
            threshold,
        }
    }

    // Walked from the end so the Vec is bounded by the tail length rather than
    // by however much a failing tool managed to emit before it gave up.
    fn tail(stdout: &str) -> String {
        let mut lines = stdout
            .lines()
            .rev()
            .take(STDOUT_TAIL_LINES)
            .collect::<Vec<_>>();
        lines.reverse();
        lines.join("\n")
    }
}

impl Gate for CrapGate<'_> {
    fn label(&self) -> String {
        String::from("CRAP")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled {
                binary: String::from(BINARY),
            });
        }

        let mut args = vec![
            String::from("crap4rust"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }
        args.extend([
            String::from("--warn-only"),
            String::from("--threshold"),
            self.threshold.clone(),
            String::from("--output-format"),
            String::from("json"),
        ]);

        let outcome = self.runner.run_capturing("cargo", &args)?;

        // crap4rust exits non-zero the moment any function reaches the
        // threshold, --warn-only notwithstanding, so the exit code alone
        // cannot separate "found something" from "failed to run". The parsed
        // report decides; the exit code and output only colour a parse
        // failure, which is the case where the tool genuinely did not report.
        let report =
            self.parser
                .parse(outcome.stdout())
                .map_err(|failure| GateFailure::Unreported {
                    failure,
                    exit_code: outcome.exit_code(),
                    stderr: outcome.stderr().to_string(),
                    stdout_tail: Self::tail(outcome.stdout()),
                })?;

        println!("{}", report.summary());

        if report.is_clean() {
            return Ok(());
        }

        for offender in report.offenders() {
            println!("  {}", offender.describe());
        }

        Err(GateFailure::Crappy {
            count: report.crappy_functions(),
        })
    }
}
