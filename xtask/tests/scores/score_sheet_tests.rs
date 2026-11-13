// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::scores::crate_score::CrateScore;
use xtask::scores::score_sheet::ScoreSheet;

fn crate_score(name: &str, grip: f64, braintax: f64) -> CrateScore {
    CrateScore {
        name: String::from(name),
        grip,
        braintax,
    }
}

// Each crate's grip over its braintax, and their ratio; then the sums and
// theirs -- the tools' versions named, since scores of different releases do
// not compare.
#[test]
fn render_lists_each_crate_and_the_total_with_their_ratios() {
    // Arrange
    let sheet = ScoreSheet::new(
        String::from("0.10.0"),
        String::from("0.14.0"),
        vec![
            crate_score("core", 107.38, 844.21),
            crate_score("spec-model", 4.05, 27.65),
        ],
    );

    // Act
    let rendered = sheet.render();

    // Assert
    assert_eq!(rendered, "Grip / braintax, crates/ (cargo-grip4rust 0.10.0, cargo-braintax4rust 0.14.0):\n  core              107.38 /   844.21  0.1272\n  spec-model          4.05 /    27.65  0.1465\n  total             111.43 /   871.86  0.1278");
}

// A crate whose functions cost nothing has no ratio to show.
#[test]
fn render_of_a_crate_with_no_braintax_shows_no_ratio() {
    // Arrange
    let sheet = ScoreSheet::new(
        String::from("0.10.0"),
        String::from("0.14.0"),
        vec![crate_score("spec-model", 3.80, 0.0)],
    );

    // Act
    let rendered = sheet.render();

    // Assert
    assert_eq!(rendered, "Grip / braintax, crates/ (cargo-grip4rust 0.10.0, cargo-braintax4rust 0.14.0):\n  spec-model          3.80 /     0.00  -\n  total               3.80 /     0.00  -");
}
