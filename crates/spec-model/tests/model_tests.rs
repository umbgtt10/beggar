// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::spec_files::SpecFiles;
use crate::support::states::States;
use beggar_spec_model::answer::Answer;
use beggar_spec_model::model::Model;
use beggar_spec_model::spec_reader::SpecReader;
use serde_json::json;
use serde_json::Value;

// Node 2's incarnation 3, fresh, keeping one finished session, with no bound.
fn fresh() -> Model {
    Model::new(
        SpecReader::table(&SpecFiles::matrix()),
        SpecReader::origin(&json!({"Fresh": {
            "instance": {"node": "2", "incarnation": "3"},
            "retention": "1",
            "bound": null,
        }})),
    )
}

fn peers_session() -> Value {
    json!({"node": "1", "incarnation": "7", "count": "1"})
}

fn receipt(number: &str, output: Value) -> Value {
    json!({"number": number, "output": output})
}

#[test]
fn process_from_a_checkpoint_carries_on_from_its_next_receipt() {
    // Arrange
    let mut model = Model::new(
        SpecReader::table(&SpecFiles::matrix()),
        States::holding(vec![States::session(States::peers(1), &[1, 3])], Vec::new()),
    );

    // Act
    let answer = model.process(&json!({"ReceiveRecoveryPartial": {
        "session_id": peers_session(),
        "first": "2",
        "last": "2",
        "sender": "1",
        "receiver": "2",
    }}));

    // Assert
    assert_eq!(
        answer,
        Answer {
            row: String::from("partial-advanced"),
            receipt: receipt(
                "5",
                json!({"RecoveryAdvanced": {"held": "3", "chunks": "4"}})
            ),
        }
    );
}

#[test]
fn process_numbers_a_refusal_as_any_other_call_and_changes_nothing_else() {
    // Arrange
    let mut model = fresh();
    model.process(&json!({"StartRecovery": {"chunks": "0", "sender": "2", "receiver": "1"}}));

    // Act
    let answer = model.process(&json!({"ShowRegistry": {}}));

    // Assert
    assert_eq!(
        answer,
        Answer {
            row: String::from("show-registry"),
            receipt: receipt(
                "2",
                json!({"RegistryShown": {
                    "instance": {"node": "2", "incarnation": "3"},
                    "retention": "1",
                    "bound": null,
                    "open": [],
                    "retained": [],
                    "name_count": "0",
                    "next_receipt": "3",
                }})
            ),
        }
    );
}

#[test]
fn process_of_a_partial_completing_its_session_evicts_it_and_retains_it() {
    // Arrange
    let mut model = fresh();
    model.process(&json!({"AdoptRecovery": {
        "session_id": peers_session(),
        "chunks": "2",
        "sender": "1",
        "receiver": "2",
    }}));

    // Act
    let answers: Vec<Answer> = [
        json!({"ReceiveRecoveryPartial": {
            "session_id": peers_session(),
            "first": "1",
            "last": "2",
            "sender": "1",
            "receiver": "2",
        }}),
        json!({"ShowRecovery": {"session_id": peers_session()}}),
        json!({"ListRecoveries": {}}),
    ]
    .iter()
    .map(|command| model.process(command))
    .collect();

    // Assert
    assert_eq!(
        answers,
        [
            Answer {
                row: String::from("partial-completed"),
                receipt: receipt("2", json!({"RecoveryCompleted": {}})),
            },
            Answer {
                row: String::from("recovery-shown"),
                receipt: receipt(
                    "3",
                    json!({"RecoveryShown": {
                        "record": {
                            "session_id": peers_session(),
                            "chunks": "2",
                            "sender": "1",
                            "receiver": "2",
                            "chunk_set": ["1", "2"],
                        },
                        "finished": "2",
                    }})
                ),
            },
            Answer {
                row: String::from("recoveries-listed"),
                receipt: receipt("4", json!({"RecoveriesListed": {"sessions": []}})),
            },
        ]
    );
}

#[test]
fn process_of_a_start_on_a_fresh_instance_answers_receipt_1_with_its_first_name() {
    // Arrange
    let mut model = fresh();

    // Act
    let answer =
        model.process(&json!({"StartRecovery": {"chunks": "4", "sender": "2", "receiver": "1"}}));

    // Assert
    assert_eq!(
        answer,
        Answer {
            row: String::from("start-started"),
            receipt: receipt(
                "1",
                json!({"RecoveryStarted": {"session_id": {"node": "2", "incarnation": "3", "count": "1"}}})
            ),
        }
    );
}
