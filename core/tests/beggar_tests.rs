// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::beggar::Beggar;
use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::origin::Origin;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::received_chunks::ReceivedChunks;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;

const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};

fn fresh(bound: Option<u64>) -> Beggar<()> {
    Beggar::new(Origin::Fresh(Fresh {
        instance: INSTANCE,
        retention: 0,
        bound,
    }))
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
        received: ReceivedChunks::new(),
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
    let mut beggar: Beggar<()> = Beggar::new(Origin::Checkpoint(Checkpoint {
        instance: INSTANCE,
        retention: 0,
        bound: None,
        open: vec![record(1, 8, 1, 2)],
        retained: Vec::new(),
        name_count: 1,
        next_receipt: 10,
    }));

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
fn process_of_show_registry_answers_the_state_after_its_own_call() {
    // Arrange
    let mut beggar = Beggar::<()>::new(Origin::Fresh(Fresh {
        instance: INSTANCE,
        retention: 5,
        bound: Some(2),
    }));

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
