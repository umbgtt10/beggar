// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::crap::crap_report::CrapReport;
use crate::crap::report_failure::ReportFailure;
use serde_json::from_str;

const OBJECT_OPEN: &str = "{";

pub struct CrapReportParser;

impl CrapReportParser {
    pub fn new() -> Self {
        Self
    }

    // crap4rust runs the crate's own test suite to gather coverage before it
    // reports, so its stdout carries that run's output ahead of the JSON. The
    // report is the last object opening at column zero; nested braces in the
    // payload are indented and cannot be mistaken for it.
    //
    // Starts-with rather than equals, so a compact single-line report -- the
    // whole object on one line beginning with `{` -- is found too. The column
    // zero guarantee is what does the work either way, and it survives.
    pub fn parse(&self, stdout: &str) -> Result<CrapReport, ReportFailure> {
        let start = stdout
            .lines()
            .enumerate()
            .filter(|(_, line)| line.starts_with(OBJECT_OPEN))
            .map(|(index, _)| index)
            .last()
            .ok_or(ReportFailure::NotFound)?;

        let json_text = stdout.lines().skip(start).collect::<Vec<_>>().join("\n");

        from_str(&json_text).map_err(|error| ReportFailure::Unparseable {
            reason: error.to_string(),
        })
    }
}

impl Default for CrapReportParser {
    fn default() -> Self {
        Self::new()
    }
}
