// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::states::States;
use beggar_spec_model::call::Call;
use beggar_spec_model::command::Command;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::vocabulary::command_name::CommandName;
use beggar_spec_model::vocabulary::rejection_name::RejectionName;
use beggar_spec_model::vocabulary::speller::Speller;
use serde_json::json;
use serde_json::Value;

const EVERY: [RejectionName; 9] = [
    RejectionName::NoChunks,
    RejectionName::SelfAddressed,
    RejectionName::NotAParty,
    RejectionName::OwnSession,
    RejectionName::RegistryFull,
    RejectionName::EmptyRun,
    RejectionName::Misattributed,
    RejectionName::WrongSide,
    RejectionName::ChunkOutOfRange,
];

// The rejection encoded for a command refused in a state, which a refusal
// leaves as it was.
fn encoded(rejection: RejectionName, command: &Command, state: &InstanceState) -> Value {
    rejection.encoded(&Call {
        command,
        before: state,
        after: state,
    })
}

#[test]
fn encoded_of_a_chunk_out_of_range_carries_the_run_and_the_session_s_count() {
    // Arrange
    let state = States::holding(vec![States::session(States::peers(1), &[])], Vec::new());
    let command = Commands::partial(CommandName::ReceiveRecoveryPartial, States::peers(1), 3, 9);

    // Act
    let encoded = encoded(RejectionName::ChunkOutOfRange, &command, &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"ChunkOutOfRange": {"first": "3", "last": "9", "chunks": "4"}})
    );
}

#[test]
fn encoded_of_a_misattributed_partial_carries_the_session_s_ends() {
    // Arrange
    let state = States::holding(vec![States::session(States::peers(1), &[])], Vec::new());
    let command = Command {
        sender: Some(3),
        ..Commands::partial(CommandName::ReceiveRecoveryPartial, States::peers(1), 1, 1)
    };

    // Act
    let encoded = encoded(RejectionName::Misattributed, &command, &state);

    // Assert
    assert_eq!(
        encoded,
        json!({"Misattributed": {"sender": "1", "receiver": "2"}})
    );
}

#[test]
fn encoded_of_a_rejection_carrying_nothing_is_the_empty_object() {
    // Arrange
    let state = States::instance();
    let command = Commands::start(0, 2, 2);

    // Act
    let encoded: Vec<Value> = [
        RejectionName::NoChunks,
        RejectionName::SelfAddressed,
        RejectionName::NotAParty,
        RejectionName::OwnSession,
        RejectionName::WrongSide,
    ]
    .into_iter()
    .map(|rejection| encoded(rejection, &command, &state))
    .collect();

    // Assert
    assert_eq!(
        encoded,
        [
            json!({"NoChunks": {}}),
            json!({"SelfAddressed": {}}),
            json!({"NotAParty": {}}),
            json!({"OwnSession": {}}),
            json!({"WrongSide": {}}),
        ]
    );
}

#[test]
fn encoded_of_an_empty_run_carries_the_run() {
    // Arrange
    let state = States::instance();
    let command = Commands::partial(CommandName::SendRecoveryPartial, States::peers(1), 5, 3);

    // Act
    let encoded = encoded(RejectionName::EmptyRun, &command, &state);

    // Assert
    assert_eq!(encoded, json!({"EmptyRun": {"first": "5", "last": "3"}}));
}

#[test]
fn encoded_of_registry_full_carries_the_bound_as_its_capacity() {
    // Arrange
    let state = States::instance();
    let command = Commands::start(4, 2, 1);

    // Act
    let encoded = encoded(RejectionName::RegistryFull, &command, &state);

    // Assert
    assert_eq!(encoded, json!({"RegistryFull": {"capacity": "2"}}));
}

#[test]
fn name_of_each_rejection_is_the_vocabulary_s_in_the_rejections_order() {
    // Arrange & Act
    let names: Vec<&str> = EVERY
        .iter()
        .map(|word| Speller::name(&RejectionName::SPELLING, *word))
        .collect();

    // Assert
    assert_eq!(
        names,
        [
            "NoChunks",
            "SelfAddressed",
            "NotAParty",
            "OwnSession",
            "RegistryFull",
            "EmptyRun",
            "Misattributed",
            "WrongSide",
            "ChunkOutOfRange",
        ]
    );
}

#[test]
#[should_panic(expected = "a rejection")]
fn of_a_name_no_rejection_has_panics() {
    // Arrange & Act & Assert
    let _ = Speller::of(&RejectionName::SPELLING, "Refused");
}

#[test]
fn of_each_rejection_s_name_reads_that_rejection() {
    // Arrange & Act
    let read: Vec<RejectionName> = EVERY
        .iter()
        .map(|word| {
            Speller::of(
                &RejectionName::SPELLING,
                Speller::name(&RejectionName::SPELLING, *word),
            )
        })
        .collect();

    // Assert
    assert_eq!(read, EVERY);
}
