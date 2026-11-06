// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use xtask::gates::gate::Gate;
use xtask::gates::stern_gate::SternGate;

fn gate(runner: &FakeCommandRunner) -> SternGate<'_> {
    SternGate::new(
        runner,
        String::from("Cargo.toml"),
        vec![String::from("beggar")],
    )
}

#[test]
fn label_names_the_house_rules_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "House rules");
}

#[test]
fn run_passes_the_manifest_path_and_package_to_the_tool() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0],
        vec![
            String::from("stern4rust"),
            String::from("--manifest-path"),
            String::from("Cargo.toml"),
            String::from("--package"),
            String::from("beggar"),
        ]
    );
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
fn run_with_exit_code_one_reports_a_failure_to_run() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("could not run")));
}

// 2 is a rule broken and 1 is the tool failing to start. Collapsing them would
// let a bad manifest read as a clean codebase.
#[test]
fn run_with_exit_code_two_reports_a_broken_rule() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(2));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from("a house coding rule was broken"))
    );
}

#[test]
fn run_with_the_tool_missing_returns_an_install_hint() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("cargo install cargo-stern4rust")));
}
