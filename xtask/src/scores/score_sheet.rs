// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scores::crate_score::CrateScore;

const TOTAL: &str = "total";

// The library's crates scored: each one's grip over its braintax and their
// ratio, then the sums and theirs, under a title naming both tools' releases
// -- scores of different releases do not compare.
pub struct ScoreSheet {
    grip_version: String,
    braintax_version: String,
    crates: Vec<CrateScore>,
}

impl ScoreSheet {
    pub fn new(grip_version: String, braintax_version: String, crates: Vec<CrateScore>) -> Self {
        Self {
            grip_version,
            braintax_version,
            crates,
        }
    }

    pub fn render(&self) -> String {
        let grip: f64 = self.crates.iter().map(|score| score.grip).sum();
        let braintax: f64 = self.crates.iter().map(|score| score.braintax).sum();
        let mut lines = vec![format!(
            "Grip / braintax, crates/ (cargo-grip4rust {}, cargo-braintax4rust {}):",
            self.grip_version, self.braintax_version
        )];
        lines.extend(
            self.crates
                .iter()
                .map(|score| Self::row(&score.name, score.grip, score.braintax)),
        );
        lines.push(Self::row(TOTAL, grip, braintax));
        lines.join("\n")
    }

    // A crate whose functions cost nothing has no ratio.
    fn row(name: &str, grip: f64, braintax: f64) -> String {
        let ratio = if braintax > 0.0 {
            format!("{:.4}", grip / braintax)
        } else {
            String::from("-")
        };
        format!("  {name:<16}{grip:>8.2} / {braintax:>8.2}  {ratio}")
    }
}
