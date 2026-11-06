// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::crap::crap_report_parser::CrapReportParser;

fn report_json(crappy: u32, verdicts: &[&str]) -> String {
    let functions = verdicts
        .iter()
        .map(|verdict| {
            format!(
                r#"{{"name":"f","relative_file":"src/a.rs","line":1,"complexity":1,"coverage":1.0,"crap_score":0.0,"verdict":"{verdict}"}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\n  \"total_functions\": {},\n  \"crappy_functions\": {crappy},\n  \"crappy_percent\": 0.0,\n  \"functions\": [{functions}]\n}}",
        verdicts.len()
    )
}

#[test]
fn is_clean_with_a_crappy_function_returns_false() {
    // Arrange
    let json = report_json(1, &["Crappy"]);
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let clean = report.is_clean();

    // Assert
    assert!(!clean);
}

#[test]
fn is_clean_with_no_crappy_functions_returns_true() {
    // Arrange
    let json = report_json(0, &["Clean"]);
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let clean = report.is_clean();

    // Assert
    assert!(clean);
}

#[test]
fn offenders_returns_only_the_functions_that_are_not_clean() {
    // Arrange
    let json = report_json(2, &["Clean", "Crappy", "Warning"]);
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let offenders = report.offenders();

    // Assert
    assert_eq!(offenders.len(), 2);
    assert!(offenders.iter().all(|function| !function.is_clean()));
}

#[test]
fn offenders_with_every_function_clean_returns_empty() {
    // Arrange
    let json = report_json(0, &["Clean", "Clean"]);
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let offenders = report.offenders();

    // Assert
    assert!(offenders.is_empty());
}

#[test]
fn summary_names_the_crappy_count_against_the_total() {
    // Arrange
    let json = report_json(1, &["Crappy", "Clean"]);
    let report = CrapReportParser::new().parse(&json).expect("parses");

    // Act
    let summary = report.summary();

    // Assert
    assert!(summary.contains("1/2 functions crappy"));
}
