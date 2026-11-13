// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::process::command_runner::CommandRunner;
use crate::scores::crate_score::CrateScore;
use crate::scores::library_files::LibraryFiles;
use crate::scores::score_failure::ScoreFailure;
use crate::scores::score_sheet::ScoreSheet;
use crate::scores::tool_score::ToolScore;
use crate::scores::tool_score_parser::ToolScoreParser;

const BRAINTAX: &str = "braintax4rust";
const BRAINTAX_TOOL: &str = "cargo-braintax4rust";
const BRAINTAX_TOTAL: &str = "total_braintax";
const CARGO: &str = "cargo";
const GIT: &str = "git";
const GRIP: &str = "grip4rust";
const GRIP_TOOL: &str = "cargo-grip4rust";
const GRIP_TOTAL: &str = "grip_absolute_total";
const LIBRARY: [&str; 1] = ["core"];
const LIBRARY_DIRECTORY: &str = "crates";

// The library's crates under crates/ -- core, the one beggar publishes --
// scored as they are staged: the files under them that the index holds -- the
// one git hands a commit hook -- exported into a scratch directory, so nothing
// unstaged is scored, and each crate with source there scored by
// cargo-grip4rust and cargo-braintax4rust.
pub struct LibraryScorer<'a> {
    runner: &'a dyn CommandRunner,
    scratch: String,
}

impl<'a> LibraryScorer<'a> {
    pub fn new(runner: &'a dyn CommandRunner, scratch: String) -> Self {
        Self { runner, scratch }
    }

    pub fn score(&self) -> Result<ScoreSheet, ScoreFailure> {
        let directories: Vec<String> = LIBRARY
            .iter()
            .map(|name| format!("{LIBRARY_DIRECTORY}/{name}"))
            .collect();
        let mut listing_args = vec!["ls-files", "-z", "--"];
        listing_args.extend(directories.iter().map(String::as_str));
        let listing = self.capture(GIT, &listing_args)?;
        let files: Vec<String> = listing
            .split('\0')
            .filter(|file| !file.is_empty())
            .map(String::from)
            .collect();
        let crates = LibraryFiles::crates(LIBRARY_DIRECTORY, &LIBRARY, &files);
        if crates.is_empty() {
            return Err(ScoreFailure::NoCrates);
        }
        let prefix = format!("--prefix={}/", self.scratch);
        let mut export = vec!["checkout-index", "--force", prefix.as_str(), "--"];
        export.extend(files.iter().map(String::as_str));
        self.capture(GIT, &export)?;
        let mut scores = Vec::new();
        let mut grip_version = String::new();
        let mut braintax_version = String::new();
        for name in crates {
            let directory = format!("{}/{LIBRARY_DIRECTORY}/{name}", self.scratch);
            let grip = self.measured(GRIP, GRIP_TOOL, GRIP_TOTAL, &directory)?;
            let braintax = self.measured(BRAINTAX, BRAINTAX_TOOL, BRAINTAX_TOTAL, &directory)?;
            scores.push(CrateScore {
                name,
                grip: grip.total,
                braintax: braintax.total,
            });
            grip_version = grip.version;
            braintax_version = braintax.version;
        }
        Ok(ScoreSheet::new(grip_version, braintax_version, scores))
    }

    fn capture(&self, program: &str, args: &[&str]) -> Result<String, ScoreFailure> {
        let args: Vec<String> = args.iter().map(|arg| String::from(*arg)).collect();
        let outcome = self.runner.run_capturing(program, &args)?;
        if !outcome.is_success() {
            return Err(ScoreFailure::CommandFailed {
                command: format!("{program} {}", args.join(" ")),
                stderr: String::from(outcome.stderr()),
            });
        }
        Ok(String::from(outcome.stdout()))
    }

    fn measured(
        &self,
        subcommand: &str,
        tool: &str,
        total: &str,
        directory: &str,
    ) -> Result<ToolScore, ScoreFailure> {
        let report = self.capture(CARGO, &[subcommand, "--json", directory])?;
        ToolScoreParser::parse(tool, &report, total)
    }
}
