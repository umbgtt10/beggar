// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::spec_files::SpecFiles;
use crate::support::states::States;
use beggar_spec_model::chunk_set::ChunkSet;
use beggar_spec_model::command::Command;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::retained::Retained;
use beggar_spec_model::run::Run;
use beggar_spec_model::session::Session;
use beggar_spec_model::spec_reader::SpecReader;
use beggar_spec_model::table::row::Row;
use beggar_spec_model::vocabulary::command_name::CommandName;
use beggar_spec_model::vocabulary::output_name::OutputName;
use beggar_spec_model::vocabulary::rejection_name::RejectionName;
use serde_json::json;

#[test]
fn command_of_a_cancel_keeps_its_reason_as_given() {
    // Arrange
    let value = json!({"CancelRecovery": {
        "session_id": {"node": "1", "incarnation": "7", "count": "1"},
        "reason": {"Stalled": {"held": "3"}},
    }});

    // Act
    let command = SpecReader::command(&value);

    // Assert
    assert_eq!(
        command,
        Command {
            reason: Some(json!({"Stalled": {"held": "3"}})),
            ..Commands::naming(CommandName::CancelRecovery, States::peers(1))
        }
    );
}

#[test]
fn command_of_a_partial_reads_its_name_run_and_ends() {
    // Arrange
    let value = json!({"ReceiveRecoveryPartial": {
        "session_id": {"node": "1", "incarnation": "7", "count": "2"},
        "first": "3",
        "last": "18446744073709551615",
        "sender": "1",
        "receiver": "2",
    }});

    // Act
    let command = SpecReader::command(&value);

    // Assert
    assert_eq!(
        command,
        Commands::partial(
            CommandName::ReceiveRecoveryPartial,
            States::peers(2),
            3,
            u64::MAX
        )
    );
}

#[test]
fn command_of_a_query_carrying_nothing_reads_nothing_but_its_name() {
    // Arrange
    let value = json!({"ShowRegistry": {}});

    // Act
    let command = SpecReader::command(&value);

    // Assert
    assert_eq!(command, Commands::of(CommandName::ShowRegistry));
}

#[test]
#[should_panic(expected = "a command: an object of one member")]
fn command_of_a_value_that_is_not_an_object_panics() {
    // Arrange & Act & Assert
    let _ = SpecReader::command(&json!("ShowRegistry"));
}

#[test]
fn command_of_an_adopt_reads_its_name_count_and_ends() {
    // Arrange
    let value = json!({"AdoptRecovery": {
        "session_id": {"node": "1", "incarnation": "7", "count": "5"},
        "chunks": "4",
        "sender": "1",
        "receiver": "2",
    }});

    // Act
    let command = SpecReader::command(&value);

    // Assert
    assert_eq!(command, Commands::adopt(States::peers(5), 4, 1, 2));
}

#[test]
fn origin_of_a_checkpoint_is_the_state_it_describes() {
    // Arrange
    let value = json!({"Checkpoint": {
        "instance": {"node": "2", "incarnation": "3"},
        "retention": "1",
        "bound": "2",
        "open": [{
            "session_id": {"node": "1", "incarnation": "7", "count": "2"},
            "chunks": "4",
            "sender": "1",
            "receiver": "2",
            "chunk_set": ["2", "3"],
        }],
        "retained": [{
            "record": {
                "session_id": {"node": "1", "incarnation": "7", "count": "1"},
                "chunks": "4",
                "sender": "1",
                "receiver": "2",
                "chunk_set": ["1", "3"],
            },
            "finished": "4",
        }],
        "name_count": "2",
        "next_receipt": "5",
    }});

    // Act
    let state = SpecReader::origin(&value);

    // Assert
    assert_eq!(
        state,
        States::holding(
            vec![States::session(States::peers(2), &[2, 3])],
            vec![States::retained(States::peers(1), 4)],
        )
    );
}

#[test]
fn origin_of_a_checkpoint_reads_a_chunk_set_at_the_top_of_u64() {
    // Arrange
    let value = json!({"Checkpoint": {
        "instance": {"node": "2", "incarnation": "3"},
        "retention": "1",
        "bound": "2",
        "open": [{
            "session_id": {"node": "1", "incarnation": "7", "count": "1"},
            "chunks": "18446744073709551615",
            "sender": "1",
            "receiver": "2",
            "chunk_set": ["18446744073709551615"],
        }],
        "retained": [],
        "name_count": "2",
        "next_receipt": "5",
    }});

    // Act
    let state = SpecReader::origin(&value);

    // Assert
    assert_eq!(
        state.open,
        [Session {
            name: States::peers(1),
            chunks: u64::MAX,
            sender: 1,
            receiver: 2,
            chunk_set: ChunkSet::default().with(Run {
                first: u64::MAX,
                last: u64::MAX,
            }),
        }]
    );
    assert_eq!(state.retained, Vec::<Retained>::new());
}

#[test]
fn origin_of_a_fresh_instance_holds_nothing_and_numbers_its_first_receipt_1() {
    // Arrange
    let value = json!({"Fresh": {
        "instance": {"node": "2", "incarnation": "18446744073709551615"},
        "retention": "0",
        "bound": null,
    }});

    // Act
    let state = SpecReader::origin(&value);

    // Assert
    assert_eq!(
        state,
        InstanceState {
            node: 2,
            incarnation: u64::MAX,
            retention: 0,
            bound: None,
            open: Vec::new(),
            retained: Vec::new(),
            name_count: 0,
            next_receipt: 1,
        }
    );
}

#[test]
fn table_of_a_matrix_reads_a_row_with_no_rejection_as_none() {
    // Arrange
    let matrix = json!({"version": "0.6.0", "rows": [{
        "id": "show-registry",
        "commands": ["ShowRegistry"],
        "situation": "always: the checkpoint, describing the state after the call",
        "output": "RegistryShown",
    }]});

    // Act
    let table = SpecReader::table(&matrix);

    // Assert
    assert_eq!(
        table.row(
            &Commands::of(CommandName::ShowRegistry),
            &States::instance()
        ),
        &Row {
            id: String::from("show-registry"),
            commands: vec![CommandName::ShowRegistry],
            output: OutputName::RegistryShown,
            rejection: None,
        }
    );
}

#[test]
fn table_of_the_matrix_answers_a_start_of_no_chunks_by_its_first_row() {
    // Arrange
    let matrix = SpecFiles::matrix();

    // Act
    let table = SpecReader::table(&matrix);

    // Assert
    assert_eq!(
        table.row(&Commands::start(0, 2, 2), &States::instance()),
        &Row {
            id: String::from("no-chunks"),
            commands: vec![CommandName::StartRecovery, CommandName::AdoptRecovery],
            output: OutputName::RecoveryRejected,
            rejection: Some(RejectionName::NoChunks),
        }
    );
}
