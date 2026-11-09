// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use xtask::gates::coverage_gate::CoverageGate;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;

fn every_tier(runner: &FakeCommandRunner) -> CoverageGate<'_> {
    CoverageGate::new(
        runner,
        String::from("Cargo.toml"),
        vec![String::from("beggar"), String::from("beggar-ladder")],
        None,
    )
}

fn one_tier(runner: &FakeCommandRunner) -> CoverageGate<'_> {
    CoverageGate::new(
        runner,
        String::from("Cargo.toml"),
        vec![String::from("beggar"), String::from("beggar-ladder")],
        Some(String::from("conformance")),
    )
}

#[test]
fn label_of_every_tier_names_it() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = every_tier(&runner).label();

    // Assert
    assert_eq!(label, "Coverage, every tier");
}

#[test]
fn label_of_one_tier_names_it_alone() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = one_tier(&runner).label();

    // Assert
    assert_eq!(label, "Coverage, the conformance tier alone");
}

#[test]
fn run_asks_llvm_cov_to_fail_on_any_line_region_or_function_left_uncovered() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = every_tier(&runner).run();

    // Assert
    assert_eq!(runner.programs(), vec![String::from("cargo")]);
    assert_eq!(
        runner.calls()[0],
        [
            "llvm-cov",
            "--manifest-path",
            "Cargo.toml",
            "--package",
            "beggar",
            "--package",
            "beggar-ladder",
            "--summary-only",
            "--fail-uncovered-lines",
            "0",
            "--fail-uncovered-regions",
            "0",
            "--fail-uncovered-functions",
            "0",
        ]
        .map(String::from)
    );
}

#[test]
fn run_of_one_tier_runs_its_tests_alone() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = one_tier(&runner).run();

    // Assert
    let args = &runner.calls()[0];
    assert_eq!(
        args[args.len() - 2..],
        [String::from("--"), String::from("conformance::")]
    );
}

#[test]
fn run_with_a_non_zero_exit_code_reports_what_the_tier_leaves_uncovered() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = one_tier(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Uncovered {
            scope: String::from("the conformance tier alone"),
        })
    );
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(0));

    // Act
    let result = every_tier(&runner).run();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_without_cargo_llvm_cov_reports_it_not_installed() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = every_tier(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::NotInstalled {
            binary: String::from("cargo-llvm-cov"),
        })
    );
}
