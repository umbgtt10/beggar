// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use xtask::process::launch_failure::LaunchFailure;
use xtask::scores::crate_score::CrateScore;
use xtask::scores::library_scorer::LibraryScorer;
use xtask::scores::score_failure::ScoreFailure;
use xtask::scores::score_sheet::ScoreSheet;

const LISTING: &str =
    "crates/core/src/lib.rs\0crates/core/Cargo.toml\0crates/core/src/registry.rs\0";

fn answering_the_core() -> FakeCommandRunner {
    FakeCommandRunner::new().with_replies(&[
        LISTING,
        "",
        r#"{"version":"0.10.0","overall":{"grip_absolute_total":107.38}}"#,
        r#"{"version":"0.14.0","overall":{"total_braintax":844.21}}"#,
    ])
}

fn arguments(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| String::from(*arg)).collect()
}

#[test]
fn score_exports_the_staged_library_files_then_scores_each_crate_there() {
    // Arrange
    let runner = answering_the_core();
    let scorer = LibraryScorer::new(&runner, String::from("/scratch"));

    // Act
    scorer.score().expect("scored");

    // Assert
    assert_eq!(
        runner.calls()[..4],
        [
            arguments(&["ls-files", "-z", "--", "crates/core",]),
            arguments(&[
                "checkout-index",
                "--force",
                "--prefix=/scratch/",
                "--",
                "crates/core/src/lib.rs",
                "crates/core/Cargo.toml",
                "crates/core/src/registry.rs",
            ]),
            arguments(&["grip4rust", "--json", "/scratch/crates/core"]),
            arguments(&["braintax4rust", "--json", "/scratch/crates/core"]),
        ]
    );
    assert_eq!(runner.programs()[..3], ["git", "git", "cargo"]);
}

#[test]
fn score_of_a_failing_tool_names_its_command_with_what_it_said() {
    // Arrange
    let runner = answering_the_core().with_capturing_codes(&[Some(0), Some(0), Some(1)]);
    let scorer = LibraryScorer::new(&runner, String::from("/scratch"));

    // Act
    let scored = scorer.score();

    // Assert
    assert_eq!(
        scored.err(),
        Some(ScoreFailure::CommandFailed {
            command: String::from("cargo grip4rust --json /scratch/crates/core"),
            stderr: String::new(),
        })
    );
}

#[test]
fn score_of_a_runner_that_cannot_launch_fails_with_the_launch() {
    // Arrange
    let runner = FakeCommandRunner::new().with_launch_failure();
    let scorer = LibraryScorer::new(&runner, String::from("/scratch"));

    // Act
    let scored = scorer.score();

    // Assert
    assert_eq!(
        scored.err(),
        Some(ScoreFailure::Launch(LaunchFailure::new(
            "git",
            "refused by the fake"
        )))
    );
}

#[test]
fn score_scores_each_library_crate_as_staged() {
    // Arrange
    let runner = answering_the_core();
    let scorer = LibraryScorer::new(&runner, String::from("/scratch"));

    // Act
    let scored = scorer.score().map(|sheet| sheet.render());

    // Assert
    let expected = ScoreSheet::new(
        String::from("0.10.0"),
        String::from("0.14.0"),
        vec![CrateScore {
            name: String::from("core"),
            grip: 107.38,
            braintax: 844.21,
        }],
    );
    assert_eq!(scored, Ok(expected.render()));
}

#[test]
fn score_with_no_library_crate_staged_fails_saying_so() {
    // Arrange
    let runner = FakeCommandRunner::new().with_replies(&[""]);
    let scorer = LibraryScorer::new(&runner, String::from("/scratch"));

    // Act
    let scored = scorer.score();

    // Assert
    assert_eq!(scored.err(), Some(ScoreFailure::NoCrates));
}
