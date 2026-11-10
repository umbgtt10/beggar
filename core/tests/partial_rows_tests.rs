// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::chunk_set::ChunkSet;
use beggar::output::Output;
use beggar::partial_rows::PartialRows;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use beggar::side::Side;

const NODE: u64 = 1;

fn name(count: u64) -> SessionId {
    SessionId {
        node: NODE,
        incarnation: 7,
        count,
    }
}

fn open() -> Vec<SessionRecord> {
    vec![
        SessionRecord {
            session_id: name(1),
            chunks: 4,
            sender: 2,
            receiver: NODE,
            chunk_set: ChunkSet::new(),
        },
        SessionRecord {
            session_id: name(2),
            chunks: 4,
            sender: NODE,
            receiver: 2,
            chunk_set: ChunkSet::new(),
        },
    ]
}

#[test]
fn attribution_of_a_receive_in_range_on_its_receivers_end_is_its_sessions_place() {
    // Arrange
    let open = open();
    let rows = PartialRows::new(NODE, &open);

    // Act
    let attribution = rows.attribution(Side::Receiving, name(1), 1, 4, 2, NODE);

    // Assert
    assert_eq!(attribution, Ok(0));
}

#[test]
fn attribution_of_a_receive_on_the_senders_end_is_wrong_side_before_its_range() {
    // Arrange
    let open = open();
    let rows = PartialRows::new(NODE, &open);

    // Act
    let attribution = rows.attribution(Side::Receiving, name(2), 0, 9, NODE, 2);

    // Assert
    assert_eq!(
        attribution,
        Err(Output::RecoveryRejected(Rejection::WrongSide))
    );
}

#[test]
fn attribution_of_a_send_for_no_open_session_is_not_started() {
    // Arrange
    let open = open();
    let rows = PartialRows::new(NODE, &open);

    // Act
    let attribution = rows.attribution(Side::Sending, name(3), 1, 1, 9, 8);

    // Assert
    assert_eq!(attribution, Err(Output::RecoveryNotStarted));
}

#[test]
fn attribution_of_a_send_from_zero_or_past_the_chunks_is_chunk_out_of_range() {
    // Arrange
    let open = open();
    let rows = PartialRows::new(NODE, &open);

    // Act
    let from_zero = rows.attribution(Side::Sending, name(2), 0, 1, NODE, 2);
    let past_the_chunks = rows.attribution(Side::Sending, name(2), 4, 5, NODE, 2);

    // Assert
    assert_eq!(
        from_zero,
        Err(Output::RecoveryRejected(Rejection::ChunkOutOfRange {
            first: 0,
            last: 1,
            chunks: 4
        }))
    );
    assert_eq!(
        past_the_chunks,
        Err(Output::RecoveryRejected(Rejection::ChunkOutOfRange {
            first: 4,
            last: 5,
            chunks: 4
        }))
    );
}

#[test]
fn attribution_of_a_send_in_range_on_its_senders_end_is_its_sessions_place() {
    // Arrange
    let open = open();
    let rows = PartialRows::new(NODE, &open);

    // Act
    let attribution = rows.attribution(Side::Sending, name(2), 1, 4, NODE, 2);

    // Assert
    assert_eq!(attribution, Ok(1));
}

#[test]
fn attribution_of_a_send_on_the_receivers_end_is_wrong_side_before_its_range() {
    // Arrange
    let open = open();
    let rows = PartialRows::new(NODE, &open);

    // Act
    let attribution = rows.attribution(Side::Sending, name(1), 0, 9, 2, NODE);

    // Assert
    assert_eq!(
        attribution,
        Err(Output::RecoveryRejected(Rejection::WrongSide))
    );
}

#[test]
fn attribution_of_a_send_with_other_ends_is_misattributed_before_its_side() {
    // Arrange
    let open = open();
    let rows = PartialRows::new(NODE, &open);

    // Act
    let attribution = rows.attribution(Side::Sending, name(1), 1, 1, NODE, 2);

    // Assert
    assert_eq!(
        attribution,
        Err(Output::RecoveryRejected(Rejection::Misattributed {
            sender: 2,
            receiver: NODE
        }))
    );
}

#[test]
fn attribution_of_an_empty_run_is_empty_run_before_its_session_is_looked_for() {
    // Arrange
    let open = open();
    let rows = PartialRows::new(NODE, &open);

    // Act
    let attribution = rows.attribution(Side::Sending, name(3), 2, 1, 9, 8);

    // Assert
    assert_eq!(
        attribution,
        Err(Output::RecoveryRejected(Rejection::EmptyRun {
            first: 2,
            last: 1
        }))
    );
}
