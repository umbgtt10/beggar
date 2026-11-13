// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::process::launch_failure::LaunchFailure;
use xtask::scores::score_failure::ScoreFailure;

#[test]
fn from_of_a_launch_failure_holds_it() {
    // Arrange
    let launch = LaunchFailure::new("git", "no such program");

    // Act
    let failure = ScoreFailure::from(launch.clone());

    // Assert
    assert_eq!(failure, ScoreFailure::Launch(launch));
}

#[test]
fn to_string_of_a_failed_command_names_it_and_carries_its_stderr() {
    // Arrange
    let failure = ScoreFailure::CommandFailed {
        command: String::from("cargo grip4rust --json /scratch/core"),
        stderr: String::from("error: no such directory"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "cargo grip4rust --json /scratch/core failed: error: no such directory"
    );
}

#[test]
fn to_string_of_an_unreadable_report_names_the_tool_and_why() {
    // Arrange
    let failure = ScoreFailure::Unreadable {
        tool: String::from("cargo-braintax4rust"),
        reason: String::from("no overall.total_braintax"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "cargo-braintax4rust's report is unreadable: no overall.total_braintax"
    );
}

#[test]
fn to_string_of_no_crates_says_no_library_crate_is_staged() {
    // Arrange & Act
    let text = ScoreFailure::NoCrates.to_string();

    // Assert
    assert_eq!(text, "no crate under crates/ has a staged source file");
}
