// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_spec_model::rules::change::Change;
use beggar_spec_model::rules::registry_situation::RegistrySituation;
use beggar_spec_model::rules::rule::Rule;
use beggar_spec_model::rules::rule_book::RuleBook;
use beggar_spec_model::rules::session_situation::SessionSituation;
use beggar_spec_model::rules::situation::Situation;
use std::collections::BTreeSet;

#[test]
fn row_ids_names_each_known_row_once_in_the_matrix_s_order() {
    // Arrange & Act
    let row_ids = RuleBook::row_ids();

    // Assert
    let distinct: BTreeSet<&str> = row_ids.iter().copied().collect();
    assert_eq!(row_ids.len(), 25);
    assert_eq!(distinct.len(), 25);
    assert_eq!(row_ids.first(), Some(&"no-chunks"));
    assert_eq!(row_ids.last(), Some(&"show-registry"));
}

#[test]
#[should_panic(expected = "a row the model knows")]
fn rule_of_a_row_the_model_does_not_know_panics() {
    // Arrange & Act & Assert
    let _ = RuleBook::rule("start-refused");
}

#[test]
fn rule_of_adopt_registry_full_applies_to_a_name_not_open_alone() {
    // Arrange & Act
    let rule = RuleBook::rule("adopt-registry-full");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::Registry(RegistrySituation::FullAndNotOpen),
            change: Change::Nothing,
        }
    );
}

#[test]
fn rule_of_cancel_cancelled_cancels_the_open_session() {
    // Arrange & Act
    let rule = RuleBook::rule("cancel-cancelled");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::Registry(RegistrySituation::Open),
            change: Change::SessionCancelled,
        }
    );
}

#[test]
fn rule_of_partial_completed_completes_the_session_its_run_fills() {
    // Arrange & Act
    let rule = RuleBook::rule("partial-completed");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::Session(SessionSituation::Completes),
            change: Change::SessionCompleted,
        }
    );
}
