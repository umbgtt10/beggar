// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::crap::report_failure::ReportFailure;

#[test]
fn to_string_of_a_missing_report_says_none_was_found() {
    // Arrange
    let failure = ReportFailure::NotFound;

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(shown, "could not find a JSON report in crap4rust's output");
}

#[test]
fn to_string_of_an_unparseable_report_carries_the_reason() {
    // Arrange
    let failure = ReportFailure::Unparseable {
        reason: String::from("expected value at line 1 column 1"),
    };

    // Act
    let shown = failure.to_string();

    // Assert
    assert_eq!(
        shown,
        "could not parse crap4rust JSON: expected value at line 1 column 1"
    );
}
