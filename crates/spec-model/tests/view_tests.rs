// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::states::States;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::retained::Retained;
use beggar_spec_model::run::Run;
use beggar_spec_model::view::View;
use serde_json::json;

#[test]
fn checkpoint_of_a_state_lists_its_records_in_canonical_order() {
    // Arrange
    let state = States::holding(
        vec![
            States::session(States::peers(2), &[]),
            States::session(States::peers(1), &[4]),
        ],
        vec![
            Retained {
                session: States::session(States::peers(3), &[1, 2, 3, 4]),
                finished: 2,
            },
            States::retained(States::peers(3), 1),
            States::retained(States::own(1), 3),
        ],
    );

    // Act
    let checkpoint = View::checkpoint(&state);

    // Assert
    let record = |node: &str, incarnation: &str, count: &str, chunk_set: &[&str]| {
        json!({
            "session_id": {"node": node, "incarnation": incarnation, "count": count},
            "chunks": "4",
            "sender": "1",
            "receiver": "2",
            "chunk_set": chunk_set,
        })
    };
    assert_eq!(
        checkpoint,
        json!({
            "instance": {"node": "2", "incarnation": "3"},
            "retention": "1",
            "bound": "2",
            "open": [record("1", "7", "1", &["4"]), record("1", "7", "2", &[])],
            "retained": [
                {"record": record("1", "7", "3", &["1", "3"]), "finished": "1"},
                {"record": record("1", "7", "3", &["1", "2", "3", "4"]), "finished": "2"},
                {"record": record("2", "3", "1", &["1", "3"]), "finished": "3"},
            ],
            "name_count": "2",
            "next_receipt": "5",
        })
    );
}

#[test]
fn checkpoint_of_a_state_with_no_bound_writes_it_null() {
    // Arrange
    let state = InstanceState {
        bound: None,
        ..States::instance()
    };

    // Act
    let checkpoint = View::checkpoint(&state);

    // Assert
    assert_eq!(checkpoint["bound"], json!(null));
}

#[test]
fn names_of_a_state_are_its_open_sessions_names_in_canonical_order() {
    // Arrange
    let state = States::holding(
        vec![
            States::session(States::own(1), &[]),
            States::session(States::peers(10), &[]),
            States::session(States::peers(9), &[]),
        ],
        vec![States::retained(States::peers(1), 4)],
    );

    // Act
    let names = View::names(&state);

    // Assert
    assert_eq!(
        names,
        json!([
            {"node": "1", "incarnation": "7", "count": "9"},
            {"node": "1", "incarnation": "7", "count": "10"},
            {"node": "2", "incarnation": "3", "count": "1"},
        ])
    );
}

#[test]
fn number_of_the_top_of_u64_is_its_decimal_digits() {
    // Arrange & Act
    let written = View::number(u64::MAX);

    // Assert
    assert_eq!(written, json!("18446744073709551615"));
}

#[test]
fn record_of_a_session_lists_its_chunk_set_as_ids() {
    // Arrange
    let session = States::session(States::peers(1), &[4, 1, 2]);

    // Act
    let record = View::record(&session);

    // Assert
    assert_eq!(
        record,
        json!({
            "session_id": {"node": "1", "incarnation": "7", "count": "1"},
            "chunks": "4",
            "sender": "1",
            "receiver": "2",
            "chunk_set": ["1", "2", "4"],
        })
    );
}

#[test]
fn run_of_a_run_of_one_writes_its_first_and_last() {
    // Arrange & Act
    let written = View::run(Run { first: 7, last: 7 });

    // Assert
    assert_eq!(written, json!({"first": "7", "last": "7"}));
}

#[test]
fn runs_of_a_list_writes_each_run_s_first_and_last() {
    // Arrange
    let runs = [Run { first: 1, last: 3 }, Run { first: 5, last: 5 }];

    // Act
    let written = View::runs(&runs);

    // Assert
    assert_eq!(
        written,
        json!([{"first": "1", "last": "3"}, {"first": "5", "last": "5"}])
    );
}

#[test]
fn session_name_of_a_name_writes_its_three_numbers() {
    // Arrange & Act
    let written = View::session_name(States::peers(18_446_744_073_709_551_615));

    // Assert
    assert_eq!(
        written,
        json!({"node": "1", "incarnation": "7", "count": "18446744073709551615"})
    );
}
