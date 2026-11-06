// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::process::command_runner::CommandRunner;
use xtask::process::system_command_runner::SystemCommandRunner;

#[test]
fn is_available_for_a_binary_on_the_path_returns_true() {
    // Arrange
    let runner = SystemCommandRunner::new();

    // Act
    let available = runner.is_available("cargo");

    // Assert
    assert!(available, "cargo runs this test, so it is on the PATH");
}

#[test]
fn is_available_for_a_binary_that_does_not_exist_returns_false() {
    // Arrange
    let runner = SystemCommandRunner::new();

    // Act
    let available = runner.is_available("cargo-no-such-subcommand-4rust");

    // Assert
    assert!(!available);
}

#[test]
fn run_capturing_a_program_that_does_not_exist_returns_a_launch_error() {
    // Arrange
    let runner = SystemCommandRunner::new();

    // Act
    let result = runner.run_capturing("no-such-program-4rust", &[]);

    // Assert
    assert!(
        result.is_err_and(|error| error.to_string().starts_with("failed to launch")),
        "a missing program is a launch failure, not an exit code"
    );
}

#[test]
fn run_capturing_a_succeeding_command_returns_its_stdout_and_zero() {
    // Arrange
    let runner = SystemCommandRunner::new();
    let args = vec![String::from("--version")];

    // Act
    let outcome = runner.run_capturing("cargo", &args).expect("cargo runs");

    // Assert
    assert_eq!(outcome.exit_code(), Some(0));
    assert!(outcome.stdout().starts_with("cargo "));
}

#[test]
fn run_streaming_a_succeeding_command_returns_zero() {
    // Arrange
    let runner = SystemCommandRunner::new();
    let args = vec![String::from("--version")];

    // Act
    let code = runner.run_streaming("cargo", &args).expect("cargo runs");

    // Assert
    assert_eq!(code, Some(0));
}
