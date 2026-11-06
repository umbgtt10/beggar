// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use xtask::gates::gate::Gate;
use xtask::gates::iceberg_gate::IcebergGate;

fn gate(runner: &FakeCommandRunner) -> IcebergGate<'_> {
    IcebergGate::new(
        runner,
        String::from("Cargo.toml"),
        vec![String::from("beggar")],
        String::from("5"),
    )
}

#[test]
fn label_names_the_file_risk_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "File risk");
}

// The ceiling must reach the CLI as it was written. Formatting a float here
// would render it with the current locale's separator, so a fractional ceiling
// of 9.5 would arrive as `9,5` where the comma is the decimal mark, and that
// does not parse. This repository's ceiling is a whole number, so nothing would
// go wrong today -- the string is what keeps that true after the first edit to
// a fractional one.
#[test]
fn run_passes_the_threshold_through_unchanged() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    let call = &runner.calls()[0];
    assert!(call.contains(&String::from("--threshold")));
    assert!(call.contains(&String::from("5")));
    assert_eq!(runner.programs(), vec![String::from("cargo")]);
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_exit_code_one_reports_the_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("exit code")));
}

// 2 is iceberg4rust's own "offenders found", which has to read as a breached
// ceiling rather than as the tool failing to run.
#[test]
fn run_with_exit_code_two_names_the_ceiling_it_breached() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(2));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from("a file is at or above the ceiling of 5"))
    );
}

#[test]
fn run_with_the_tool_missing_returns_an_install_hint() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error
        .to_string()
        .contains("cargo install cargo-iceberg4rust")));
}
