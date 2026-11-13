// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::checkpoint::Checkpoint;
use beggar::chunk_set::ChunkSet;
use beggar::instance_id::InstanceId;
use beggar::output::Output;
use beggar::registry::Registry;
use beggar::rejection::Rejection;
use beggar::retained_record::RetainedRecord;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use beggar::side::Side;

const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};

fn empty(bound: Option<u64>) -> Checkpoint {
    Checkpoint {
        instance: INSTANCE,
        retention: 0,
        bound,
        open: Vec::new(),
        retained: Vec::new(),
        name_count: 2,
        next_receipt: 5,
    }
}

fn kept(session_id: SessionId, held: &[(u64, u64)], finished: u64) -> RetainedRecord {
    let mut chunk_set = ChunkSet::new();
    for (first, last) in held {
        chunk_set.add(*first, *last);
    }
    RetainedRecord {
        record: SessionRecord {
            session_id,
            chunks: 4,
            sender: INSTANCE.node,
            receiver: 2,
            chunk_set,
        },
        finished,
    }
}

fn name(node: u64, incarnation: u64, count: u64) -> SessionId {
    SessionId {
        node,
        incarnation,
        count,
    }
}

fn retained_names(registry: &Registry) -> Vec<(SessionId, u64)> {
    registry
        .checkpoint()
        .retained
        .iter()
        .map(|kept| (kept.record.session_id, kept.finished))
        .collect()
}

fn retaining(retention: u64) -> Checkpoint {
    Checkpoint {
        retention,
        ..empty(None)
    }
}

#[test]
fn account_of_a_run_adding_chunks_advances_carrying_held_and_chunks() {
    // Arrange
    let mut registry = Registry::new(empty(None));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);

    // Act
    let output = registry.account::<()>(0, 1, 2);

    // Assert
    assert_eq!(output, Output::RecoveryAdvanced { held: 2, chunks: 4 });
    assert_eq!(registry.checkpoint().open[0].chunk_set.ids(), vec![1, 2]);
}

#[test]
fn account_of_a_run_already_held_is_unaffected() {
    // Arrange
    let mut registry = Registry::new(empty(None));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.account::<()>(0, 1, 2);

    // Act
    let output = registry.account::<()>(0, 2, 2);

    // Assert
    assert_eq!(output, Output::RecoveryUnaffected);
}

#[test]
fn account_of_the_run_filling_the_set_at_a_retention_keeps_it_finished_by_the_call_counted_last() {
    // Arrange
    let mut registry = Registry::new(retaining(1));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.account::<()>(0, 3, 4);
    registry.count_call();

    // Act
    let output = registry.account::<()>(0, 1, 2);

    // Assert
    assert_eq!(output, Output::RecoveryCompleted);
    let checkpoint = registry.checkpoint();
    assert!(checkpoint.open.is_empty());
    assert_eq!(checkpoint.retained, vec![kept(name(1, 7, 1), &[(1, 4)], 5)]);
}

#[test]
fn account_of_the_run_filling_the_set_completes_and_evicts_the_session() {
    // Arrange
    let mut registry = Registry::new(empty(None));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.account::<()>(0, 3, 4);

    // Act
    let output = registry.account::<()>(0, 1, 2);

    // Assert
    assert_eq!(output, Output::RecoveryCompleted);
    assert!(registry.checkpoint().open.is_empty());
    assert!(registry.checkpoint().retained.is_empty());
}

#[test]
fn count_call_numbers_each_call_one_more_than_the_last() {
    // Arrange
    let mut registry = Registry::new(empty(None));

    // Act
    let numbers = [registry.count_call(), registry.count_call()];

    // Assert
    assert_eq!(numbers, [5, 6]);
    assert_eq!(registry.checkpoint().next_receipt, 7);
}

#[test]
fn evict_at_a_retention_keeps_the_session_as_it_stood_finished_by_the_call_counted_last() {
    // Arrange
    let mut registry = Registry::new(retaining(3));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.account::<()>(0, 2, 2);
    registry.count_call();
    registry.count_call();

    // Act
    let evicted = registry.evict(name(1, 7, 1));

    // Assert
    assert!(evicted);
    let checkpoint = registry.checkpoint();
    assert!(checkpoint.open.is_empty());
    assert_eq!(checkpoint.retained, vec![kept(name(1, 7, 1), &[(2, 2)], 6)]);
}

#[test]
fn evict_beyond_the_retention_lets_the_earliest_finished_go_whatever_its_name() {
    // Arrange
    let mut registry = Registry::new(retaining(2));
    for count in 1..=3 {
        registry.open(name(1, 7, count), 4, INSTANCE.node, 2);
    }
    registry.count_call();
    registry.evict(name(1, 7, 2));
    registry.count_call();
    registry.evict(name(1, 7, 1));
    registry.count_call();

    // Act
    registry.evict(name(1, 7, 3));

    // Assert
    assert_eq!(
        retained_names(&registry),
        vec![(name(1, 7, 1), 6), (name(1, 7, 3), 7)]
    );
}

#[test]
fn evict_keeps_the_retained_in_the_order_of_their_names_then_of_their_finishing() {
    // Arrange
    let mut registry = Registry::new(retaining(3));
    registry.open(name(2, 3, 1), 4, INSTANCE.node, 2);
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.count_call();
    registry.evict(name(2, 3, 1));
    registry.count_call();
    registry.evict(name(1, 7, 1));
    registry.open(name(2, 3, 1), 4, INSTANCE.node, 2);
    registry.count_call();

    // Act
    registry.evict(name(2, 3, 1));

    // Assert
    assert_eq!(
        retained_names(&registry),
        vec![(name(1, 7, 1), 6), (name(2, 3, 1), 5), (name(2, 3, 1), 7)]
    );
}

#[test]
fn evict_of_an_open_session_removes_it_and_is_true() {
    // Arrange
    let mut registry = Registry::new(empty(None));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.open(name(1, 7, 2), 4, INSTANCE.node, 2);

    // Act
    let evicted = registry.evict(name(1, 7, 1));

    // Assert
    assert!(evicted);
    assert!(registry.checkpoint().retained.is_empty());
    let names: Vec<SessionId> = registry
        .checkpoint()
        .open
        .iter()
        .map(|record| record.session_id)
        .collect();
    assert_eq!(names, vec![name(1, 7, 2)]);
}

#[test]
fn evict_of_no_open_session_is_false() {
    // Arrange
    let mut registry = Registry::new(empty(None));

    // Act
    let evicted = registry.evict(name(1, 7, 1));

    // Assert
    assert!(!evicted);
}

#[test]
fn mint_names_the_next_session_after_this_instance() {
    // Arrange
    let mut registry = Registry::new(empty(None));

    // Act
    let session_id = registry.mint();

    // Assert
    assert_eq!(session_id, name(1, 7, 3));
    assert_eq!(registry.checkpoint().name_count, 3);
}

#[test]
fn open_keeps_the_records_in_the_order_of_their_names() {
    // Arrange
    let mut registry = Registry::new(empty(None));

    // Act
    registry.open(name(2, 3, 1), 4, INSTANCE.node, 2);
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.open(name(3, 1, 5), 4, INSTANCE.node, 2);

    // Assert
    let names: Vec<SessionId> = registry
        .checkpoint()
        .open
        .iter()
        .map(|record| record.session_id)
        .collect();
    assert_eq!(names, vec![name(1, 7, 1), name(2, 3, 1), name(3, 1, 5)]);
}

#[test]
fn opening_rows_count_the_open_sessions_against_the_bound() {
    // Arrange
    let mut registry = Registry::new(empty(Some(1)));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);

    // Act
    let refusal = registry.opening_rows().start_refusal(4, INSTANCE.node, 2);

    // Assert
    assert_eq!(refusal, Some(Rejection::RegistryFull { capacity: 1 }));
}

#[test]
fn partial_rows_find_the_open_sessions() {
    // Arrange
    let mut registry = Registry::new(empty(None));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);

    // Act
    let attribution = registry.partial_rows().attribution::<()>(
        Side::Sending,
        name(1, 7, 1),
        1,
        1,
        INSTANCE.node,
        2,
    );

    // Assert
    assert_eq!(attribution, Ok(0));
}

#[test]
fn query_rows_read_the_open_and_the_retained_sessions() {
    // Arrange
    let mut registry = Registry::new(retaining(1));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.open(name(1, 7, 2), 4, INSTANCE.node, 2);
    registry.count_call();
    registry.evict(name(1, 7, 1));

    // Act
    let outputs = [
        registry.query_rows().recoveries::<()>(),
        registry.query_rows().recovery::<()>(name(1, 7, 1)),
    ];

    // Assert
    assert_eq!(
        outputs,
        [
            Output::RecoveriesListed {
                sessions: vec![name(1, 7, 2)]
            },
            Output::RecoveryShown {
                record: kept(name(1, 7, 1), &[], 5).record,
                finished: Some(5)
            },
        ]
    );
}
