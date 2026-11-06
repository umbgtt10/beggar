// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::process::launch_failure::LaunchFailure;

#[test]
fn to_string_names_the_program_and_the_reason() {
    // Arrange
    let failure = LaunchFailure::new("cargo", "No such file or directory (os error 2)");

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(
        shown,
        "failed to launch cargo: No such file or directory (os error 2)"
    );
}
