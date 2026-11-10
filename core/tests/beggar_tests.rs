// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::recording_observer::RecordingObserver;
use beggar::beggar::Beggar;
use beggar::checkpoint::Checkpoint;
use beggar::chunk_set::ChunkSet;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::no_op_observer::NoOpObserver;
use beggar::origin::Origin;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use std::cell::RefCell;
use std::rc::Rc;

const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};

fn fresh(bound: Option<u64>) -> Beggar<()> {
    Beggar::new(
        Origin::Fresh(Fresh {
            instance: INSTANCE,
            retention: 0,
            bound,
        }),
        Box::new(NoOpObserver),
    )
}

fn name(count: u64) -> SessionId {
    SessionId {
        node: INSTANCE.node,
        incarnation: INSTANCE.incarnation,
        count,
    }
}

fn record(count: u64, chunks: u64, sender: u64, receiver: u64) -> SessionRecord {
    SessionRecord {
        session_id: name(count),
        chunks,
        sender,
        receiver,
        chunk_set: ChunkSet::new(),
    }
}

fn send(count: u64, first: u64, last: u64, sender: u64, receiver: u64) -> Command {
    Command::SendRecoveryPartial {
        session_id: name(count),
        first,
        last,
        sender,
        receiver,
    }
}

fn shown(beggar: &mut Beggar<()>) -> Checkpoint {
    match beggar.process(Command::ShowRegistry).output {
        Output::RegistryShown(checkpoint) => checkpoint,
        other => panic!("ShowRegistry answered {other:?}"),
    }
}

fn start(chunks: u64, sender: u64, receiver: u64) -> Command {
    Command::StartRecovery {
        chunks,
        sender,
        receiver,
    }
}

#[test]
fn new_from_a_checkpoint_carries_on_from_its_next_receipt_and_its_next_name() {
    // Arrange
    let mut beggar: Beggar<()> = Beggar::new(
        Origin::Checkpoint(Checkpoint {
            instance: INSTANCE,
            retention: 0,
            bound: None,
            open: vec![record(1, 8, 1, 2)],
            retained: Vec::new(),
            name_count: 1,
            next_receipt: 10,
        }),
        Box::new(NoOpObserver),
    );

    // Act
    let receipt = beggar.process(start(4, 1, 3));

    // Assert
    assert_eq!(
        receipt,
        Receipt {
            number: 10,
            output: Output::RecoveryStarted {
                session_id: name(2)
            },
        }
    );
}

#[test]
fn process_of_a_partial_already_held_answers_unaffected() {
    // Arrange
    let mut beggar = fresh(None);
    beggar.process(start(6, 1, 2));
    beggar.process(send(1, 2, 4, 1, 2));

    // Act
    let receipt = beggar.process(send(1, 3, 4, 1, 2));

    // Assert
    assert_eq!(receipt.output, Output::RecoveryUnaffected);
    assert_eq!(shown(&mut beggar).open[0].chunk_set.ids(), vec![2, 3, 4]);
}

#[test]
fn process_of_a_partial_beyond_the_chunks_answers_chunk_out_of_range_and_adds_nothing() {
    // Arrange
    let mut beggar = fresh(None);
    beggar.process(start(4, 1, 2));

    // Act
    let receipt = beggar.process(send(1, 3, 5, 1, 2));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryRejected(Rejection::ChunkOutOfRange {
            first: 3,
            last: 5,
            chunks: 4
        })
    );
    assert!(shown(&mut beggar).open[0].chunk_set.ids().is_empty());
}

#[test]
fn process_of_a_partial_filling_the_set_completes_and_evicts_the_session() {
    // Arrange
    let mut beggar = fresh(None);
    beggar.process(start(4, 1, 2));
    beggar.process(send(1, 3, 4, 1, 2));

    // Act
    let receipt = beggar.process(send(1, 1, 2, 1, 2));

    // Assert
    assert_eq!(receipt.output, Output::RecoveryCompleted);
    let checkpoint = shown(&mut beggar);
    assert!(checkpoint.open.is_empty());
    assert_eq!(checkpoint.name_count, 1);
}

#[test]
fn process_of_a_partial_for_no_open_session_answers_not_started() {
    // Arrange
    let mut beggar = fresh(None);
    beggar.process(start(4, 1, 2));

    // Act
    let receipt = beggar.process(send(2, 1, 1, 1, 2));

    // Assert
    assert_eq!(receipt.output, Output::RecoveryNotStarted);
}

#[test]
fn process_of_a_partial_on_the_receivers_end_answers_wrong_side() {
    // Arrange
    let mut beggar = fresh(None);
    beggar.process(start(4, 2, 1));

    // Act
    let receipt = beggar.process(send(1, 1, 2, 2, 1));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryRejected(Rejection::WrongSide)
    );
}

#[test]
fn process_of_a_partial_with_new_chunks_advances_carrying_held_and_chunks() {
    // Arrange
    let mut beggar = fresh(None);
    beggar.process(start(8, 1, 2));

    // Act
    let receipt = beggar.process(send(1, 1, 3, 1, 2));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryAdvanced { held: 3, chunks: 8 }
    );
    assert_eq!(shown(&mut beggar).open[0].chunk_set.ids(), vec![1, 2, 3]);
}

#[test]
fn process_of_a_partial_with_other_ends_answers_misattributed_carrying_the_sessions() {
    // Arrange
    let mut beggar = fresh(None);
    beggar.process(start(4, 1, 2));

    // Act
    let receipt = beggar.process(send(1, 1, 1, 2, 1));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryRejected(Rejection::Misattributed {
            sender: 1,
            receiver: 2
        })
    );
}

#[test]
fn process_of_a_start_at_the_bound_answers_registry_full_carrying_the_bound() {
    // Arrange
    let mut beggar = fresh(Some(1));
    beggar.process(start(8, 1, 2));

    // Act
    let receipt = beggar.process(start(5, 1, 3));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryRejected(Rejection::RegistryFull { capacity: 1 })
    );
}

#[test]
fn process_of_a_start_between_two_other_nodes_answers_not_a_party() {
    // Arrange
    let mut beggar = fresh(None);

    // Act
    let receipt = beggar.process(start(4, 2, 3));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryRejected(Rejection::NotAParty)
    );
}

#[test]
fn process_of_a_start_on_a_fresh_instance_opens_it_under_the_first_name() {
    // Arrange
    let mut beggar = fresh(None);

    // Act
    let receipt = beggar.process(start(8, 1, 2));

    // Assert
    assert_eq!(
        receipt,
        Receipt {
            number: 1,
            output: Output::RecoveryStarted {
                session_id: name(1)
            },
        }
    );
    assert_eq!(shown(&mut beggar).open, vec![record(1, 8, 1, 2)]);
}

#[test]
fn process_of_a_start_with_no_chunks_answers_no_chunks_and_mints_no_name() {
    // Arrange
    let mut beggar = fresh(None);

    // Act
    let receipt = beggar.process(start(0, 1, 2));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryRejected(Rejection::NoChunks)
    );
    let checkpoint = shown(&mut beggar);
    assert!(checkpoint.open.is_empty());
    assert_eq!(checkpoint.name_count, 0);
}

#[test]
fn process_of_a_start_with_one_end_twice_answers_self_addressed() {
    // Arrange
    let mut beggar = fresh(None);

    // Act
    let receipt = beggar.process(start(4, 1, 1));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryRejected(Rejection::SelfAddressed)
    );
}

#[test]
fn process_of_an_empty_run_answers_empty_run_before_looking_for_its_session() {
    // Arrange
    let mut beggar = fresh(None);

    // Act
    let receipt = beggar.process(send(1, 3, 2, 1, 2));

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryRejected(Rejection::EmptyRun { first: 3, last: 2 })
    );
}

#[test]
fn process_of_every_call_numbers_its_receipt_one_more_than_the_last() {
    // Arrange
    let mut beggar = fresh(None);

    // Act
    let numbers: Vec<u64> = [start(0, 1, 2), start(8, 1, 2), Command::ShowRegistry]
        .into_iter()
        .map(|command| beggar.process(command).number)
        .collect();

    // Assert
    assert_eq!(numbers, vec![1, 2, 3]);
}

#[test]
fn process_of_every_call_reports_it_and_its_receipt_to_the_observer() {
    // Arrange
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut beggar: Beggar<()> = Beggar::new(
        Origin::Fresh(Fresh {
            instance: INSTANCE,
            retention: 0,
            bound: Some(1),
        }),
        Box::new(RecordingObserver::new(Rc::clone(&calls))),
    );
    let commands = vec![
        start(0, 1, 2),
        start(8, 1, 2),
        start(4, 1, 3),
        Command::ShowRegistry,
    ];

    // Act
    let answered: Vec<(Command, Receipt)> = commands
        .into_iter()
        .map(|command| (command.clone(), beggar.process(command)))
        .collect();

    // Assert
    assert_eq!(answered.len(), 4);
    assert_eq!(*calls.borrow(), answered);
}

#[test]
fn process_of_show_registry_answers_the_state_after_its_own_call() {
    // Arrange
    let mut beggar = Beggar::<()>::new(
        Origin::Fresh(Fresh {
            instance: INSTANCE,
            retention: 5,
            bound: Some(2),
        }),
        Box::new(NoOpObserver),
    );

    // Act
    let receipt = beggar.process(Command::ShowRegistry);

    // Assert
    assert_eq!(
        receipt,
        Receipt {
            number: 1,
            output: Output::RegistryShown(Checkpoint {
                instance: INSTANCE,
                retention: 5,
                bound: Some(2),
                open: Vec::new(),
                retained: Vec::new(),
                name_count: 0,
                next_receipt: 2,
            }),
        }
    );
}
