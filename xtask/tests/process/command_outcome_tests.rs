// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::process::command_outcome::CommandOutcome;

#[test]
fn is_success_with_a_non_zero_exit_code_returns_false() {
    // Arrange
    let outcome = CommandOutcome::new(Some(101), String::new(), String::new());

    // Act & Assert
    assert!(!outcome.is_success());
}

#[test]
fn is_success_with_a_zero_exit_code_returns_true() {
    // Arrange
    let outcome = CommandOutcome::new(Some(0), String::new(), String::new());

    // Act & Assert
    assert!(outcome.is_success());
}

// A signalled process carries no exit code at all, which is a failure rather
// than an unknown to be waved through.
#[test]
fn is_success_with_no_exit_code_returns_false() {
    // Arrange
    let outcome = CommandOutcome::new(None, String::new(), String::new());

    // Act & Assert
    assert!(!outcome.is_success());
}

#[test]
fn new_holds_the_values_it_was_given() {
    // Arrange & Act
    let outcome = CommandOutcome::new(Some(2), String::from("out"), String::from("err"));

    // Assert
    assert_eq!(outcome.exit_code(), Some(2));
    assert_eq!(outcome.stdout(), "out");
    assert_eq!(outcome.stderr(), "err");
}

#[test]
fn new_with_no_exit_code_holds_none() {
    // Arrange & Act
    let outcome = CommandOutcome::new(None, String::new(), String::new());

    // Assert
    assert_eq!(outcome.exit_code(), None);
}
