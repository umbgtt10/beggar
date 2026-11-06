// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Each way a gate fails, said exactly as the run has always said it, so typing
// the failures changed what a test can match and not what a reader sees.

use xtask::crap::report_failure::ReportFailure;
use xtask::gates::gate_failure::GateFailure;
use xtask::process::launch_failure::LaunchFailure;

#[test]
fn from_a_launch_failure_wraps_it() {
    // Arrange
    let launch = LaunchFailure::new("cargo", "denied");

    // Act
    let failure = GateFailure::from(launch.clone());

    // Assert
    assert_eq!(failure, GateFailure::Launch(launch));
}

#[test]
fn to_string_of_a_broken_rule_says_so() {
    // Arrange & Act
    let shown = GateFailure::RuleBroken.to_string();

    // Assert
    assert_eq!(shown, "a house coding rule was broken");
}

#[test]
fn to_string_of_a_failure_to_launch_is_the_launch_failures() {
    // Arrange
    let failure = GateFailure::Launch(LaunchFailure::new("cargo", "denied"));

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "failed to launch cargo: denied");
}

#[test]
fn to_string_of_a_file_above_the_ceiling_names_the_ceiling() {
    // Arrange
    let failure = GateFailure::AboveCeiling {
        ceiling: String::from("5"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "a file is at or above the ceiling of 5");
}

#[test]
fn to_string_of_a_missing_tool_says_how_to_install_it() {
    // Arrange
    let failure = GateFailure::NotInstalled {
        binary: String::from("cargo-twin4rust"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(
        shown,
        "cargo-twin4rust is not installed -- run: cargo install cargo-twin4rust"
    );
}

#[test]
fn to_string_of_a_tool_that_could_not_run_names_its_exit_code() {
    // Arrange
    let failure = GateFailure::CouldNotRun {
        exit_code: Some(101),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "could not run, exit code Some(101)");
}

#[test]
fn to_string_of_added_duplication_names_the_baseline() {
    // Arrange
    let failure = GateFailure::DuplicationAdded {
        baseline: String::from("dry4rust-baseline.json"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "duplication was added beyond dry4rust-baseline.json");
}

#[test]
fn to_string_of_an_unread_report_carries_what_the_tool_left() {
    // Arrange
    let failure = GateFailure::Unreported {
        failure: ReportFailure::NotFound,
        exit_code: Some(101),
        stderr: String::from("error: cargo llvm-cov failed"),
        stdout_tail: String::from("line 1\nline 2"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(
        shown,
        "could not find a JSON report in crap4rust's output (exit code Some(101))\n\
         stderr: error: cargo llvm-cov failed\nstdout tail:\nline 1\nline 2"
    );
}

// Singular when there is one: the reason is the gate's failure line, and "1
// crappy functions detected" reads as a bug in the gate rather than as a
// finding about the code.
#[test]
fn to_string_of_crappy_functions_agrees_with_their_number() {
    // Arrange
    let one = GateFailure::Crappy { count: 1 };
    let three = GateFailure::Crappy { count: 3 };

    // Act
    let shown = [one.to_string(), three.to_string()];

    // Assert
    assert_eq!(
        shown,
        [
            String::from("1 crappy function detected"),
            String::from("3 crappy functions detected")
        ]
    );
}

#[test]
fn to_string_of_rejected_dependencies_names_the_checks() {
    // Arrange
    let failure = GateFailure::Rejected {
        checks: String::from("advisories, licenses"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "cargo-deny rejected: advisories, licenses");
}

#[test]
fn to_string_of_sources_without_a_mirror_says_so() {
    // Arrange & Act
    let shown = GateFailure::Unmirrored.to_string();

    // Assert
    assert_eq!(shown, "source files without a mirrored test");
}
