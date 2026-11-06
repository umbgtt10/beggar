// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::gates::gate_failure::GateFailure;
use xtask::gates::stage_failure::StageFailure;

#[test]
fn to_string_names_the_gate_and_its_failure() {
    // Arrange
    let failure = StageFailure::new("House rules", GateFailure::RuleBroken);

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "House rules (a house coding rule was broken)");
}
