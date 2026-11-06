// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;

const CLEAN_VERDICT: &str = "Clean";

#[derive(Debug, Clone, Deserialize)]
pub struct CrapFunction {
    name: String,
    relative_file: String,
    line: u32,
    complexity: u32,
    coverage: f64,
    crap_score: f64,
    verdict: String,
}

impl CrapFunction {
    pub fn complexity(&self) -> u32 {
        self.complexity
    }

    pub fn coverage(&self) -> f64 {
        self.coverage
    }

    pub fn crap_score(&self) -> f64 {
        self.crap_score
    }

    pub fn describe(&self) -> String {
        format!(
            "{}:{} {} (complexity {}, coverage {:.0}%, crap {:.1}) [{}]",
            self.relative_file,
            self.line,
            self.name,
            self.complexity,
            self.coverage * 100.0,
            self.crap_score,
            self.verdict,
        )
    }

    pub fn is_clean(&self) -> bool {
        self.verdict == CLEAN_VERDICT
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn relative_file(&self) -> &str {
        &self.relative_file
    }

    pub fn verdict(&self) -> &str {
        &self.verdict
    }
}
