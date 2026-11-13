// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::chunk_set::ChunkSet;
use beggar::output::Output;
use beggar::query_rows::QueryRows;
use beggar::retained_record::RetainedRecord;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;

fn name(count: u64) -> SessionId {
    SessionId {
        node: 2,
        incarnation: 3,
        count,
    }
}

fn record(count: u64, held: &[(u64, u64)]) -> SessionRecord {
    let mut chunk_set = ChunkSet::new();
    for (first, last) in held {
        chunk_set.add(*first, *last);
    }
    SessionRecord {
        session_id: name(count),
        chunks: 4,
        sender: 2,
        receiver: 1,
        chunk_set,
    }
}

fn retained(count: u64, held: &[(u64, u64)], finished: u64) -> RetainedRecord {
    RetainedRecord {
        record: record(count, held),
        finished,
    }
}

#[test]
fn missing_chunks_of_a_retained_session_answers_what_it_lacked_when_it_finished() {
    // Arrange
    let retained = [retained(1, &[(2, 2)], 9)];
    let rows = QueryRows::new(&[], &retained);

    // Act
    let output = rows.missing_chunks::<()>(name(1));

    // Assert
    assert_eq!(
        output,
        Output::MissingChunksListed {
            missing: vec![1..=1, 3..=4]
        }
    );
}

#[test]
fn missing_chunks_of_an_open_session_answers_the_chunks_it_does_not_hold() {
    // Arrange
    let open = [record(1, &[(3, 3)])];
    let rows = QueryRows::new(&open, &[]);

    // Act
    let output = rows.missing_chunks::<()>(name(1));

    // Assert
    assert_eq!(
        output,
        Output::MissingChunksListed {
            missing: vec![1..=2, 4..=4]
        }
    );
}

#[test]
fn missing_chunks_of_no_session_of_the_name_answers_not_started() {
    // Arrange
    let open = [record(1, &[])];
    let retained = [retained(2, &[(1, 4)], 9)];
    let rows = QueryRows::new(&open, &retained);

    // Act
    let output = rows.missing_chunks::<()>(name(3));

    // Assert
    assert_eq!(output, Output::RecoveryNotStarted);
}

#[test]
fn recoveries_answers_the_names_of_the_open_sessions_in_order_and_none_retained() {
    // Arrange
    let open = [record(1, &[]), record(4, &[(1, 1)])];
    let retained = [retained(2, &[(1, 4)], 9)];
    let rows = QueryRows::new(&open, &retained);

    // Act
    let output = rows.recoveries::<()>();

    // Assert
    assert_eq!(
        output,
        Output::RecoveriesListed {
            sessions: vec![name(1), name(4)]
        }
    );
}

#[test]
fn recovery_of_a_name_open_and_retained_answers_the_open_one() {
    // Arrange
    let open = [record(1, &[(1, 1)])];
    let retained = [retained(1, &[(1, 4)], 9)];
    let rows = QueryRows::new(&open, &retained);

    // Act
    let output = rows.recovery::<()>(name(1));

    // Assert
    assert_eq!(
        output,
        Output::RecoveryShown {
            record: record(1, &[(1, 1)]),
            finished: None
        }
    );
}

#[test]
fn recovery_of_a_name_retained_twice_answers_the_one_finished_last() {
    // Arrange
    let retained = [retained(1, &[(1, 4)], 9), retained(1, &[(2, 3)], 12)];
    let rows = QueryRows::new(&[], &retained);

    // Act
    let output = rows.recovery::<()>(name(1));

    // Assert
    assert_eq!(
        output,
        Output::RecoveryShown {
            record: record(1, &[(2, 3)]),
            finished: Some(12)
        }
    );
}

#[test]
fn recovery_of_a_retained_session_answers_its_record_and_the_receipt_that_finished_it() {
    // Arrange
    let retained = [retained(1, &[(1, 4)], 9), retained(2, &[(1, 1)], 11)];
    let rows = QueryRows::new(&[], &retained);

    // Act
    let output = rows.recovery::<()>(name(2));

    // Assert
    assert_eq!(
        output,
        Output::RecoveryShown {
            record: record(2, &[(1, 1)]),
            finished: Some(11)
        }
    );
}

#[test]
fn recovery_of_an_open_session_answers_its_record_unfinished() {
    // Arrange
    let open = [record(1, &[]), record(2, &[(2, 3)])];
    let rows = QueryRows::new(&open, &[]);

    // Act
    let output = rows.recovery::<()>(name(2));

    // Assert
    assert_eq!(
        output,
        Output::RecoveryShown {
            record: record(2, &[(2, 3)]),
            finished: None
        }
    );
}

#[test]
fn recovery_of_no_session_of_the_name_answers_not_started() {
    // Arrange
    let open = [record(1, &[])];
    let retained = [retained(2, &[(1, 4)], 9)];
    let rows = QueryRows::new(&open, &retained);

    // Act
    let output = rows.recovery::<()>(name(3));

    // Assert
    assert_eq!(output, Output::RecoveryNotStarted);
}
