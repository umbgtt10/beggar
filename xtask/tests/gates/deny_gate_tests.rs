// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use xtask::gates::deny_gate::DenyGate;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;

fn gate(runner: &FakeCommandRunner) -> DenyGate<'_> {
    DenyGate::new(runner, String::from("/repo/Cargo.toml"))
}

#[test]
fn label_names_the_supply_chain_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "Supply chain");
}

// The database first and on its own, so the check that follows runs offline:
// a fetch that fails is then reported as one, not as an advisory finding, and
// every check judges the lock file as committed.
#[test]
fn run_fetches_the_advisory_database_then_checks_offline() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        runner.calls(),
        vec![
            vec![
                String::from("deny"),
                String::from("--manifest-path"),
                String::from("/repo/Cargo.toml"),
                String::from("fetch"),
                String::from("db"),
            ],
            vec![
                String::from("deny"),
                String::from("--manifest-path"),
                String::from("/repo/Cargo.toml"),
                String::from("--frozen"),
                String::from("check"),
            ],
        ]
    );
}

#[test]
fn run_terminated_by_a_signal_reports_no_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_streaming_code(None)
        .with_capturing_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(result, Err(GateFailure::CouldNotRun { exit_code: None }));
}

#[test]
fn run_with_an_exit_code_outside_the_bitmask_reports_the_code() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_streaming_code(Some(101))
        .with_capturing_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::CouldNotRun {
            exit_code: Some(101)
        })
    );
}

// cargo-deny's exit code is a bitmask, one bit per check that failed.
#[test]
fn run_with_exit_code_five_names_advisories_and_licenses() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_streaming_code(Some(5))
        .with_capturing_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Rejected {
            checks: String::from("advisories, licenses")
        })
    );
}

#[test]
fn run_with_exit_code_ten_names_bans_and_sources() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_streaming_code(Some(10))
        .with_capturing_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result,
        Err(GateFailure::Rejected {
            checks: String::from("bans, sources")
        })
    );
}

// The database comes over the network, and a fetch fails now and then on a
// link that is otherwise fine; three tries, then the gate says it could not run.
#[test]
fn run_with_the_advisory_database_unreachable_tries_three_times_and_checks_nothing() {
    // Arrange
    let runner = FakeCommandRunner::new().with_capturing_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    let fetches = runner
        .calls()
        .iter()
        .filter(|args| args.ends_with(&[String::from("fetch"), String::from("db")]))
        .count();
    assert_eq!(
        (result, fetches, runner.calls().len()),
        (Err(GateFailure::CouldNotRun { exit_code: Some(1) }), 3, 3)
    );
}

#[test]
fn run_with_the_tool_missing_returns_an_install_hint_and_runs_nothing() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        (result, runner.calls().len()),
        (
            Err(GateFailure::NotInstalled {
                binary: String::from("cargo-deny")
            }),
            0
        )
    );
}
