// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::states::States;
use beggar_spec_model::call::Call;
use beggar_spec_model::command::Command;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::retained::Retained;
use beggar_spec_model::vocabulary::command_name::CommandName;
use beggar_spec_model::vocabulary::output_name::OutputName;
use beggar_spec_model::vocabulary::rejection_name::RejectionName;
use beggar_spec_model::vocabulary::speller::Speller;
use serde_json::json;
use serde_json::Value;

const EVERY: [OutputName; 12] = [
    OutputName::RecoveryStarted,
    OutputName::RecoveryAdvanced,
    OutputName::RecoveryCompleted,
    OutputName::RecoveryCancelled,
    OutputName::RecoveryUnaffected,
    OutputName::RecoveryNotStarted,
    OutputName::RecoveryConflicted,
    OutputName::RecoveryRejected,
    OutputName::RecoveriesListed,
    OutputName::RecoveryShown,
    OutputName::MissingChunksListed,
    OutputName::RegistryShown,
];

// The output encoded for a command answered in a state it left as it was.
fn encoded(output: OutputName, command: &Command, state: &InstanceState) -> Value {
    output.encoded(
        None,
        &Call {
            command,
            before: state,
            after: state,
        },
    )
}

fn peers_record(count: &str, chunk_set: &[&str]) -> Value {
    json!({
        "session_id": {"node": "1", "incarnation": "7", "count": count},
        "chunks": "4",
        "sender": "1",
        "receiver": "2",
        "chunk_set": chunk_set,
    })
}

#[test]
fn encoded_of_a_cancel_carries_the_reason_given() {
    // Arrange
    let state = States::instance();
    let command = Commands::cancel(States::peers(1));

    // Act
    let encoded = encoded(OutputName::RecoveryCancelled, &command, &state);

    // Assert
    assert_eq!(encoded, json!({"RecoveryCancelled": {"Superseded": {}}}));
}

#[test]
fn encoded_of_a_conflict_carries_what_the_open_session_holds() {
    // Arrange
    let state = States::holding(vec![States::session(States::peers(1), &[])], Vec::new());
    let command = Commands::adopt(States::peers(1), 5, 1, 2);

    // Act
    let encoded = encoded(OutputName::RecoveryConflicted, &command, &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"RecoveryConflicted": {"chunks": "4", "sender": "1", "receiver": "2"}})
    );
}

#[test]
fn encoded_of_a_partial_advanced_carries_the_chunks_then_held_and_the_session_s_count() {
    // Arrange
    let before = States::holding(vec![States::session(States::peers(1), &[1])], Vec::new());
    let after = States::holding(
        vec![States::session(States::peers(1), &[1, 2, 3])],
        Vec::new(),
    );
    let command = Commands::partial(CommandName::ReceiveRecoveryPartial, States::peers(1), 2, 3);

    // Act
    let encoded = OutputName::RecoveryAdvanced.encoded(
        None,
        &Call {
            command: &command,
            before: &before,
            after: &after,
        },
    );

    // Assert
    assert_eq!(
        encoded,
        json!({"RecoveryAdvanced": {"held": "3", "chunks": "4"}})
    );
}

#[test]
fn encoded_of_a_rejection_carries_the_rejection_encoded() {
    // Arrange
    let state = States::instance();
    let command = Commands::start(0, 2, 1);

    // Act
    let encoded = OutputName::RecoveryRejected.encoded(
        Some(RejectionName::NoChunks),
        &Call {
            command: &command,
            before: &state,
            after: &state,
        },
    );

    // Assert
    assert_eq!(encoded, json!({"RecoveryRejected": {"NoChunks": {}}}));
}

#[test]
fn encoded_of_a_retained_session_shown_is_the_one_that_finished_last() {
    // Arrange
    let state = States::holding(
        Vec::new(),
        vec![
            States::retained(States::peers(1), 4),
            Retained {
                session: States::session(States::peers(1), &[1, 2, 3, 4]),
                finished: 9,
            },
        ],
    );
    let command = Commands::naming(CommandName::ShowRecovery, States::peers(1));

    // Act
    let encoded = encoded(OutputName::RecoveryShown, &command, &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"RecoveryShown": {
            "record": peers_record("1", &["1", "2", "3", "4"]),
            "finished": "9",
        }})
    );
}

#[test]
fn encoded_of_a_start_started_carries_the_name_it_minted() {
    // Arrange
    let before = States::instance();
    let after = InstanceState {
        name_count: 3,
        ..States::instance()
    };
    let command = Commands::start(4, 2, 1);

    // Act
    let encoded = OutputName::RecoveryStarted.encoded(
        None,
        &Call {
            command: &command,
            before: &before,
            after: &after,
        },
    );

    // Assert
    assert_eq!(
        encoded,
        json!({"RecoveryStarted": {"session_id": {"node": "2", "incarnation": "3", "count": "3"}}})
    );
}

#[test]
fn encoded_of_an_adopt_started_carries_the_name_adopted() {
    // Arrange
    let state = States::instance();
    let command = Commands::adopt(States::peers(5), 4, 1, 2);

    // Act
    let encoded = encoded(OutputName::RecoveryStarted, &command, &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"RecoveryStarted": {"session_id": {"node": "1", "incarnation": "7", "count": "5"}}})
    );
}

#[test]
fn encoded_of_an_open_session_shown_is_its_record_unfinished() {
    // Arrange
    let state = States::holding(
        vec![States::session(States::peers(1), &[2])],
        vec![States::retained(States::peers(1), 4)],
    );
    let command = Commands::naming(CommandName::ShowRecovery, States::peers(1));

    // Act
    let encoded = encoded(OutputName::RecoveryShown, &command, &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"RecoveryShown": {"record": peers_record("1", &["2"]), "finished": null}})
    );
}

#[test]
fn encoded_of_an_output_where_nothing_more_is_said_carries_nothing() {
    // Arrange
    let state = States::instance();
    let command = Commands::naming(CommandName::CancelRecovery, States::peers(1));

    // Act
    let encoded: Vec<Value> = [
        OutputName::RecoveryCompleted,
        OutputName::RecoveryUnaffected,
        OutputName::RecoveryNotStarted,
    ]
    .into_iter()
    .map(|output| encoded(output, &command, &state))
    .collect();

    // Assert
    assert_eq!(
        encoded,
        [
            json!({"RecoveryCompleted": {}}),
            json!({"RecoveryUnaffected": {}}),
            json!({"RecoveryNotStarted": {}}),
        ]
    );
}

#[test]
fn encoded_of_missing_chunks_lists_them_as_maximal_runs() {
    // Arrange
    let state = States::holding(vec![States::session(States::peers(1), &[2])], Vec::new());
    let command = Commands::naming(CommandName::ListMissingChunks, States::peers(1));

    // Act
    let encoded = encoded(OutputName::MissingChunksListed, &command, &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"MissingChunksListed": {"missing": [
            {"first": "1", "last": "1"},
            {"first": "3", "last": "4"},
        ]}})
    );
}

#[test]
fn encoded_of_the_recoveries_listed_names_the_open_sessions_in_canonical_order() {
    // Arrange
    let state = States::holding(
        vec![
            States::session(States::peers(2), &[]),
            States::session(States::own(1), &[]),
            States::session(States::peers(1), &[]),
        ],
        vec![States::retained(States::peers(9), 4)],
    );
    let command = Commands::of(CommandName::ListRecoveries);

    // Act
    let encoded = encoded(OutputName::RecoveriesListed, &command, &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"RecoveriesListed": {"sessions": [
            {"node": "1", "incarnation": "7", "count": "1"},
            {"node": "1", "incarnation": "7", "count": "2"},
            {"node": "2", "incarnation": "3", "count": "1"},
        ]}})
    );
}

#[test]
fn encoded_of_the_registry_shown_is_the_checkpoint_after_the_call() {
    // Arrange
    let before = States::instance();
    let after = InstanceState {
        next_receipt: 6,
        ..States::instance()
    };
    let command = Commands::of(CommandName::ShowRegistry);

    // Act
    let encoded = OutputName::RegistryShown.encoded(
        None,
        &Call {
            command: &command,
            before: &before,
            after: &after,
        },
    );

    // Assert
    assert_eq!(
        encoded,
        json!({"RegistryShown": {
            "instance": {"node": "2", "incarnation": "3"},
            "retention": "1",
            "bound": "2",
            "open": [],
            "retained": [],
            "name_count": "2",
            "next_receipt": "6",
        }})
    );
}

#[test]
fn name_of_each_output_is_the_vocabulary_s_in_the_outputs_order() {
    // Arrange & Act
    let names: Vec<&str> = EVERY
        .iter()
        .map(|word| Speller::name(&OutputName::SPELLING, *word))
        .collect();

    // Assert
    assert_eq!(
        names,
        [
            "RecoveryStarted",
            "RecoveryAdvanced",
            "RecoveryCompleted",
            "RecoveryCancelled",
            "RecoveryUnaffected",
            "RecoveryNotStarted",
            "RecoveryConflicted",
            "RecoveryRejected",
            "RecoveriesListed",
            "RecoveryShown",
            "MissingChunksListed",
            "RegistryShown",
        ]
    );
}

#[test]
#[should_panic(expected = "an output")]
fn of_a_name_no_output_has_panics() {
    // Arrange & Act & Assert
    let _ = Speller::of(&OutputName::SPELLING, "RecoveryStopped");
}

#[test]
fn of_each_output_s_name_reads_that_output() {
    // Arrange & Act
    let read: Vec<OutputName> = EVERY
        .iter()
        .map(|word| {
            Speller::of(
                &OutputName::SPELLING,
                Speller::name(&OutputName::SPELLING, *word),
            )
        })
        .collect();

    // Assert
    assert_eq!(read, EVERY);
}
