// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_gate::FakeGate;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;
use xtask::gates::stage2::Stage2;
use xtask::gates::stage_failure::StageFailure;

#[test]
fn run_reports_the_failing_gates_label_and_reason() {
    // Arrange
    let first = FakeGate::failing("House rules", GateFailure::RuleBroken);
    let gates: Vec<&dyn Gate> = vec![&first];

    // Act
    let result = Stage2::new(gates).run();

    // Assert
    assert_eq!(
        result,
        Err(StageFailure::new("House rules", GateFailure::RuleBroken))
    );
}

// stern4rust runs first precisely so the later gates never measure a layout it
// is about to reject, which only holds if a failure actually stops the run.
#[test]
fn run_with_a_failing_first_gate_does_not_run_the_later_ones() {
    // Arrange
    let first = FakeGate::failing("first", GateFailure::RuleBroken);
    let second = FakeGate::passing("second");
    let gates: Vec<&dyn Gate> = vec![&first, &second];

    // Act
    let _ = Stage2::new(gates).run();

    // Assert
    assert!(first.has_run());
    assert!(!second.has_run(), "the run stops at the first failure");
}

#[test]
fn run_with_a_failing_second_gate_still_runs_the_first() {
    // Arrange
    let first = FakeGate::passing("first");
    let second = FakeGate::failing("second", GateFailure::RuleBroken);
    let gates: Vec<&dyn Gate> = vec![&first, &second];

    // Act
    let result = Stage2::new(gates).run();

    // Assert
    assert!(first.has_run());
    assert!(second.has_run());
    assert!(result.is_err());
}

#[test]
fn run_with_every_gate_passing_returns_ok() {
    // Arrange
    let first = FakeGate::passing("first");
    let second = FakeGate::passing("second");
    let gates: Vec<&dyn Gate> = vec![&first, &second];

    // Act
    let result = Stage2::new(gates).run();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_no_gates_returns_ok() {
    // Arrange
    let gates: Vec<&dyn Gate> = Vec::new();

    // Act
    let result = Stage2::new(gates).run();

    // Assert
    assert!(result.is_ok());
}
