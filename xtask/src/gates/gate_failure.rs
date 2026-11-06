// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::crap::report_failure::ReportFailure;
use crate::process::launch_failure::LaunchFailure;
use std::fmt;

// Each way a gate fails, said exactly as the gate has always said it.
//
// A tool that exits with a code it does not document is one failure whichever
// tool it is: it could not run, so it measured nothing. A report crap4rust did
// not produce carries what the run left behind -- its exit code, its stderr, the
// tail of its stdout -- since that is the one case where the tool genuinely did
// not report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateFailure {
    AboveCeiling {
        ceiling: String,
    },
    CouldNotRun {
        exit_code: Option<i32>,
    },
    Crappy {
        count: u32,
    },
    DuplicationAdded {
        baseline: String,
    },
    Launch(LaunchFailure),
    NotInstalled {
        binary: String,
    },
    Rejected {
        checks: String,
    },
    RuleBroken,
    Unmirrored,
    Unreported {
        failure: ReportFailure,
        exit_code: Option<i32>,
        stderr: String,
        stdout_tail: String,
    },
}

impl From<LaunchFailure> for GateFailure {
    fn from(failure: LaunchFailure) -> Self {
        Self::Launch(failure)
    }
}

impl fmt::Display for GateFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AboveCeiling { ceiling } => {
                write!(formatter, "a file is at or above the ceiling of {ceiling}")
            }
            Self::CouldNotRun { exit_code } => {
                write!(formatter, "could not run, exit code {exit_code:?}")
            }
            // Singular when there is one: this is the gate's failure line, and
            // "1 crappy functions detected" reads as a bug in the gate rather
            // than as a finding about the code.
            Self::Crappy { count: 1 } => write!(formatter, "1 crappy function detected"),
            Self::Crappy { count } => write!(formatter, "{count} crappy functions detected"),
            Self::DuplicationAdded { baseline } => {
                write!(formatter, "duplication was added beyond {baseline}")
            }
            Self::Launch(failure) => write!(formatter, "{failure}"),
            Self::NotInstalled { binary } => write!(
                formatter,
                "{binary} is not installed -- run: cargo install {binary}"
            ),
            Self::Rejected { checks } => write!(formatter, "cargo-deny rejected: {checks}"),
            Self::RuleBroken => write!(formatter, "a house coding rule was broken"),
            Self::Unmirrored => write!(formatter, "source files without a mirrored test"),
            Self::Unreported {
                failure,
                exit_code,
                stderr,
                stdout_tail,
            } => write!(
                formatter,
                "{failure} (exit code {exit_code:?})\nstderr: {stderr}\nstdout tail:\n{stdout_tail}"
            ),
        }
    }
}
