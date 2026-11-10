// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::checkpoint::Checkpoint;
use beggar::instance_id::InstanceId;
use beggar::output::Output;
use beggar::registry::Registry;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
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

fn name(node: u64, incarnation: u64, count: u64) -> SessionId {
    SessionId {
        node,
        incarnation,
        count,
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
fn evict_of_an_open_session_removes_it_and_is_true() {
    // Arrange
    let mut registry = Registry::new(empty(None));
    registry.open(name(1, 7, 1), 4, INSTANCE.node, 2);
    registry.open(name(1, 7, 2), 4, INSTANCE.node, 2);

    // Act
    let evicted = registry.evict(name(1, 7, 1));

    // Assert
    assert!(evicted);
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
