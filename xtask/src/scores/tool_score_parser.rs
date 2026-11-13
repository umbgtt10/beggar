// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scores::score_failure::ScoreFailure;
use crate::scores::tool_score::ToolScore;
use serde_json::from_str;
use serde_json::Value;

// A scoring tool's `--json` report read for its release and one of its
// overall totals -- `grip_absolute_total`, `total_braintax` -- the raw sums
// over every function, not the normalized scores.
pub struct ToolScoreParser;

impl ToolScoreParser {
    pub fn parse(tool: &str, json: &str, total: &str) -> Result<ToolScore, ScoreFailure> {
        let unreadable = |reason: String| ScoreFailure::Unreadable {
            tool: String::from(tool),
            reason,
        };
        let report: Value = from_str(json).map_err(|error| unreadable(error.to_string()))?;
        let version = report["version"]
            .as_str()
            .ok_or_else(|| unreadable(String::from("no version")))?;
        let total = report["overall"][total]
            .as_f64()
            .ok_or_else(|| unreadable(format!("no overall.{total}")))?;
        Ok(ToolScore {
            version: String::from(version),
            total,
        })
    }
}
