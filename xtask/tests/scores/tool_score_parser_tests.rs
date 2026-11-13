// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::scores::score_failure::ScoreFailure;
use xtask::scores::tool_score::ToolScore;
use xtask::scores::tool_score_parser::ToolScoreParser;

#[test]
fn parse_of_a_report_reads_its_version_and_the_total_named() {
    // Arrange
    let json = r#"{"version":"0.10.0","target":"beggar","overall":{"grip_absolute_total":4.05}}"#;

    // Act
    let score = ToolScoreParser::parse("cargo-grip4rust", json, "grip_absolute_total");

    // Assert
    assert_eq!(
        score,
        Ok(ToolScore {
            version: String::from("0.10.0"),
            total: 4.05,
        })
    );
}

#[test]
fn parse_of_a_report_without_its_version_is_unreadable() {
    // Arrange
    let json = r#"{"overall":{"grip_absolute_total":4.05}}"#;

    // Act
    let score = ToolScoreParser::parse("cargo-grip4rust", json, "grip_absolute_total");

    // Assert
    assert_eq!(
        score,
        Err(ScoreFailure::Unreadable {
            tool: String::from("cargo-grip4rust"),
            reason: String::from("no version"),
        })
    );
}

#[test]
fn parse_of_a_report_without_the_total_named_is_unreadable() {
    // Arrange
    let json = r#"{"version":"0.14.0","overall":{"braintax_normalized":12.0}}"#;

    // Act
    let score = ToolScoreParser::parse("cargo-braintax4rust", json, "total_braintax");

    // Assert
    assert_eq!(
        score,
        Err(ScoreFailure::Unreadable {
            tool: String::from("cargo-braintax4rust"),
            reason: String::from("no overall.total_braintax"),
        })
    );
}

#[test]
fn parse_of_output_that_is_not_json_is_unreadable_and_names_the_tool() {
    // Arrange & Act
    let score = ToolScoreParser::parse(
        "cargo-grip4rust",
        "error: no such path",
        "grip_absolute_total",
    );

    // Assert
    assert!(
        matches!(&score, Err(ScoreFailure::Unreadable { tool, .. }) if tool == "cargo-grip4rust"),
        "{score:?}"
    );
}
