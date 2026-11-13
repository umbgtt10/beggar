// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::scores::message_scoring::MessageScoring;

const BLOCK: &str = "Grip / braintax, crates/ (cargo-grip4rust 0.10.0, cargo-braintax4rust 0.14.0):\n  total              1.00 /     2.00  0.5000";
const OLDER_BLOCK: &str = "Grip / braintax, crates/ (cargo-grip4rust 0.9.1, cargo-braintax4rust 0.13.1):\n  total              9.00 /     9.00  1.0000";

fn scoring() -> MessageScoring {
    MessageScoring::new(String::from(BLOCK))
}

#[test]
fn applied_to_a_message_already_scored_replaces_its_scores() {
    // Arrange
    let message = format!("Subject\n\nBody.\n\n{OLDER_BLOCK}\n\nCo-Authored-By: A <a@b>\n");

    // Act
    let scored = scoring().applied_to(&message);

    // Assert
    assert_eq!(
        scored,
        format!("Subject\n\nBody.\n\n{BLOCK}\n\nCo-Authored-By: A <a@b>\n")
    );
}

// A last paragraph of prose that happens to hold a colon is not trailers: a
// trailer's key is one word, letters, digits and dashes.
#[test]
fn applied_to_a_message_ending_in_prose_with_a_colon_puts_the_scores_last() {
    // Arrange
    let message = "Subject\n\nThe reason it matters: nothing else counts.\n";

    // Act
    let scored = scoring().applied_to(message);

    // Assert
    assert_eq!(
        scored,
        format!("Subject\n\nThe reason it matters: nothing else counts.\n\n{BLOCK}\n")
    );
}

// What git hands the hook of a message a commit was asked for with no words:
// left as it is, so git aborts the commit as it would.
#[test]
fn applied_to_a_message_of_comments_alone_leaves_it_as_it_is() {
    // Arrange
    let message = "\n# Please enter the commit message for your changes.\n#\n";

    // Act
    let scored = scoring().applied_to(message);

    // Assert
    assert_eq!(scored, message);
}

#[test]
fn applied_to_a_message_with_git_s_comments_puts_the_scores_above_them() {
    // Arrange
    let message = "Subject\n\n# Please enter the commit message for your changes.\n#\n";

    // Act
    let scored = scoring().applied_to(message);

    // Assert
    assert_eq!(
        scored,
        format!("Subject\n\n{BLOCK}\n\n# Please enter the commit message for your changes.\n#\n")
    );
}

#[test]
fn applied_to_a_message_with_trailers_puts_the_scores_above_them() {
    // Arrange
    let message = "Subject\n\nBody line.\n\nCo-Authored-By: A <a@b>\n";

    // Act
    let scored = scoring().applied_to(message);

    // Assert
    assert_eq!(
        scored,
        format!("Subject\n\nBody line.\n\n{BLOCK}\n\nCo-Authored-By: A <a@b>\n")
    );
}

#[test]
fn applied_to_a_message_without_trailers_puts_the_scores_last() {
    // Arrange
    let message = "Subject\n\nBody line.\n";

    // Act
    let scored = scoring().applied_to(message);

    // Assert
    assert_eq!(scored, format!("Subject\n\nBody line.\n\n{BLOCK}\n"));
}

// A subject shaped like a trailer is a subject still: the scores follow it.
#[test]
fn applied_to_a_subject_alone_puts_the_scores_after_it() {
    // Arrange
    let message = "Fix: the thing\n";

    // Act
    let scored = scoring().applied_to(message);

    // Assert
    assert_eq!(scored, format!("Fix: the thing\n\n{BLOCK}\n"));
}

// `git commit --verbose` appends the diff below a scissors line, which git
// cuts off with everything after it.
#[test]
fn applied_to_a_verbose_message_puts_the_scores_above_the_scissors() {
    // Arrange
    let message = "Subject\n# ------------------------ >8 ------------------------\n# Do not modify or remove the line above.\ndiff --git a/x b/x\n";

    // Act
    let scored = scoring().applied_to(message);

    // Assert
    assert_eq!(
        scored,
        format!("Subject\n\n{BLOCK}\n# ------------------------ >8 ------------------------\n# Do not modify or remove the line above.\ndiff --git a/x b/x\n")
    );
}
