// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::crap::crap_report_parser::CrapReportParser;

const CLEAN_REPORT: &str = r#"{
  "scope_name": "cargo-iceberg4rust",
  "total_functions": 77,
  "crappy_functions": 0,
  "crappy_percent": 0.0,
  "verdict": "Clean",
  "functions": []
}"#;

fn report_with_preamble() -> String {
    format!("running 3 tests\ntest a ... ok\n\ntest result: ok.\n\n{CLEAN_REPORT}")
}

#[test]
fn parse_a_bare_report_returns_its_totals() {
    // Arrange
    let parser = CrapReportParser::new();

    // Act
    let report = parser.parse(CLEAN_REPORT).expect("a bare report parses");

    // Assert
    assert_eq!(report.total_functions(), 77);
    assert_eq!(report.crappy_functions(), 0);
}

// The case that made two CI runs unreadable: crap4rust runs the test suite for
// coverage first, so the JSON is never at the top of stdout.
#[test]
fn parse_a_report_behind_test_output_still_finds_the_report() {
    // Arrange
    let parser = CrapReportParser::new();
    let stdout = report_with_preamble();

    // Act
    let report = parser.parse(&stdout).expect("the report is found");

    // Assert
    assert_eq!(report.total_functions(), 77);
}

// Every brace inside the payload is indented, so only the report's own opening
// sits at column zero. Anchoring on that is what lets the last-match search be
// safe rather than lucky.
#[test]
fn parse_a_report_holding_nested_objects_anchors_on_the_outermost_brace() {
    // Arrange
    let parser = CrapReportParser::new();
    let stdout = r#"noise before
{
  "total_functions": 1,
  "crappy_functions": 1,
  "crappy_percent": 100.0,
  "functions": [
    {
      "name": "f",
      "relative_file": "src/a.rs",
      "line": 3,
      "complexity": 9,
      "coverage": 0.0,
      "crap_score": 90.0,
      "verdict": "Crappy"
    }
  ]
}"#;

    // Act
    let report = parser
        .parse(stdout)
        .expect("nested objects do not confuse it");

    // Assert
    assert_eq!(report.total_functions(), 1);
    assert_eq!(report.functions().len(), 1);
    assert_eq!(report.functions()[0].name(), "f");
}

#[test]
fn parse_a_truncated_report_returns_a_parse_error() {
    // Arrange
    let parser = CrapReportParser::new();

    // Act
    let result = parser.parse("{\n  \"total_functions\": 1,");

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("could not parse crap4rust JSON")));
}

#[test]
fn parse_empty_output_returns_a_not_found_error() {
    // Arrange
    let parser = CrapReportParser::new();

    // Act
    let result = parser.parse("");

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("could not find a JSON report")));
}

#[test]
fn parse_output_carrying_no_report_returns_a_not_found_error() {
    // Arrange
    let parser = CrapReportParser::new();

    // Act
    let result = parser.parse("error: cargo llvm-cov failed with exit code Some(101)");

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("could not find a JSON report")));
}
