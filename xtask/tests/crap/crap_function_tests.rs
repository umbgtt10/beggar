// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::crap::crap_report_parser::CrapReportParser;

fn function_with_verdict(verdict: &str) -> String {
    format!(
        r#"{{
  "total_functions": 1,
  "crappy_functions": 0,
  "crappy_percent": 0.0,
  "functions": [
    {{"name":"decode","relative_file":"src/codec.rs","line":42,"complexity":7,"coverage":0.5,"crap_score":24.5,"verdict":"{verdict}"}}
  ]
}}"#
    )
}

#[test]
fn describe_names_the_file_line_and_function() {
    // Arrange
    let json = function_with_verdict("Crappy");
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let described = report.functions()[0].describe();

    // Assert
    assert!(described.starts_with("src/codec.rs:42 decode"));
}

#[test]
fn describe_renders_coverage_as_a_percentage_and_carries_the_verdict() {
    // Arrange
    let json = function_with_verdict("Crappy");
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let described = report.functions()[0].describe();

    // Assert
    assert!(described.contains("coverage 50%"));
    assert!(described.contains("[Crappy]"));
}

#[test]
fn is_clean_for_a_clean_verdict_returns_true() {
    // Arrange
    let json = function_with_verdict("Clean");
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let clean = report.functions()[0].is_clean();

    // Assert
    assert!(clean);
}

#[test]
fn is_clean_for_any_other_verdict_returns_false() {
    // Arrange
    let json = function_with_verdict("Crappy");
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let clean = report.functions()[0].is_clean();

    // Assert
    assert!(!clean);
}

#[test]
fn parse_reads_every_field_of_a_function() {
    // Arrange
    let json = function_with_verdict("Crappy");

    // Act
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Assert
    let function = &report.functions()[0];
    assert_eq!(function.name(), "decode");
    assert_eq!(function.relative_file(), "src/codec.rs");
    assert_eq!(function.line(), 42);
    assert_eq!(function.complexity(), 7);
    assert!((function.coverage() - 0.5).abs() < f64::EPSILON);
    assert!((function.crap_score() - 24.5).abs() < f64::EPSILON);
    assert_eq!(function.verdict(), "Crappy");
    assert!(report.crappy_percent().abs() < f64::EPSILON);
}
