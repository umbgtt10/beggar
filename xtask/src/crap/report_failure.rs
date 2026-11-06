// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::fmt;

// Why crap4rust's output yielded no report: no JSON object in it at all, or one
// that does not parse as the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportFailure {
    NotFound,
    Unparseable { reason: String },
}

impl fmt::Display for ReportFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(
                formatter,
                "could not find a JSON report in crap4rust's output"
            ),
            Self::Unparseable { reason } => {
                write!(formatter, "could not parse crap4rust JSON: {reason}")
            }
        }
    }
}
