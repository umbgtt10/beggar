// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use xtask::crap::crap_report_parser::CrapReportParser;
use xtask::gates::crap_gate::CrapGate;
use xtask::gates::gate::Gate;

const CLEAN_REPORT: &str = r#"{
  "total_functions": 2,
  "crappy_functions": 0,
  "crappy_percent": 0.0,
  "functions": []
}"#;
const CRAPPY_REPORT: &str = r#"{
  "total_functions": 2,
  "crappy_functions": 1,
  "crappy_percent": 50.0,
  "functions": [
    {"name":"decode","relative_file":"src/codec.rs","line":42,"complexity":9,"coverage":0.0,"crap_score":90.0,"verdict":"Crappy"}
  ]
}"#;

fn gate<'a>(runner: &'a FakeCommandRunner, parser: &'a CrapReportParser) -> CrapGate<'a> {
    CrapGate::new(
        runner,
        parser,
        String::from("Cargo.toml"),
        vec![String::from("beggar")],
        String::from("15"),
    )
}

#[test]
fn label_names_the_crap_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let parser = CrapReportParser::new();

    // Act
    let label = gate(&runner, &parser).label();

    // Assert
    assert_eq!(label, "CRAP");
}

#[test]
fn run_asks_for_json_so_the_verdict_never_depends_on_parsing_prose() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(CLEAN_REPORT);
    let parser = CrapReportParser::new();

    // Act
    let _ = gate(&runner, &parser).run();

    // Assert
    let call = &runner.calls()[0];
    assert!(call.contains(&String::from("--output-format")));
    assert!(call.contains(&String::from("json")));
    assert_eq!(runner.programs(), vec![String::from("cargo")]);
}

// crap4rust exits non-zero whenever a function reaches the threshold, so a
// clean report arriving alongside a non-zero code must still pass. Judging on
// the exit code instead would fail every run that reported anything at all.
#[test]
fn run_with_a_clean_report_and_a_non_zero_exit_code_still_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_stdout(CLEAN_REPORT)
        .with_streaming_code(Some(2));
    let parser = CrapReportParser::new();

    // Act
    let result = gate(&runner, &parser).run();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_a_clean_report_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(CLEAN_REPORT);
    let parser = CrapReportParser::new();

    // Act
    let result = gate(&runner, &parser).run();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_a_crappy_report_names_how_many_were_found() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout(CRAPPY_REPORT);
    let parser = CrapReportParser::new();

    // Act
    let result = gate(&runner, &parser).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from("1 crappy function detected"))
    );
}

// A tool that fails late can emit thousands of lines before it gives up, and
// the ones that say why are the last. Truncating from the wrong end, or
// truncating in reverse, would hand the operator the least useful thirty.
#[test]
fn run_with_a_long_unparseable_output_keeps_the_last_thirty_lines_in_order() {
    // Arrange
    let stdout = (1..=200)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let runner = FakeCommandRunner::new()
        .with_stdout(&stdout)
        .with_streaming_code(Some(101));
    let parser = CrapReportParser::new();

    // Act
    let result = gate(&runner, &parser).run();

    // Assert
    let error = result
        .expect_err("output carrying no report must fail")
        .to_string();
    assert!(error.contains("line 171\nline 172"));
    assert!(error.ends_with("line 200"));
    assert!(!error.contains("line 170"));
}

#[test]
fn run_with_no_report_in_the_output_also_reports_the_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_stdout("nothing useful")
        .with_streaming_code(Some(101));
    let parser = CrapReportParser::new();

    // Act
    let result = gate(&runner, &parser).run();

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("101")));
}

// The failure that made two CI runs unreadable: no report at all, because
// crap4rust's own coverage backend was missing.
#[test]
fn run_with_no_report_in_the_output_surfaces_the_tools_own_message() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_stdout("error: cargo llvm-cov failed with exit code Some(101)")
        .with_streaming_code(Some(2));
    let parser = CrapReportParser::new();

    // Act
    let result = gate(&runner, &parser).run();

    // Assert
    assert!(
        result.is_err_and(|error| error.to_string().contains("cargo llvm-cov failed")),
        "the tool's own stdout must reach the operator, not just an exit code"
    );
}

#[test]
fn run_with_the_tool_missing_returns_an_install_hint() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);
    let parser = CrapReportParser::new();

    // Act
    let result = gate(&runner, &parser).run();

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("cargo install cargo-crap4rust")));
}
