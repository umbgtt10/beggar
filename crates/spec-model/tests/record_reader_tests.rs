// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::states::States;
use beggar_spec_model::record_reader::RecordReader;
use serde_json::json;

#[test]
fn retained_of_a_retained_record_reads_its_record_and_its_finish() {
    // Arrange
    let value = json!({
        "record": {
            "session_id": {"node": "1", "incarnation": "7", "count": "2"},
            "chunks": "4",
            "sender": "1",
            "receiver": "2",
            "chunk_set": ["1", "3"],
        },
        "finished": "9",
    });

    // Act
    let retained = RecordReader::retained(&value);

    // Assert
    assert_eq!(retained, States::retained(States::peers(2), 9));
}

#[test]
fn session_name_of_a_name_reads_its_three_numbers() {
    // Arrange
    let value = json!({"node": "2", "incarnation": "3", "count": "5"});

    // Act
    let name = RecordReader::session_name(&value);

    // Assert
    assert_eq!(name, States::own(5));
}

#[test]
fn session_of_a_record_reads_its_chunk_set_from_its_ids() {
    // Arrange
    let value = json!({
        "session_id": {"node": "1", "incarnation": "7", "count": "1"},
        "chunks": "4",
        "sender": "1",
        "receiver": "2",
        "chunk_set": ["4", "2", "3"],
    });

    // Act
    let session = RecordReader::session(&value);

    // Assert
    assert_eq!(session, States::session(States::peers(1), &[2, 3, 4]));
}
