// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::recording_observer::RecordingObserver;
use beggar::beggar::Beggar;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::no_op_observer::NoOpObserver;
use beggar::observer::Observer;
use beggar::origin::Origin;
use beggar::receipt::Receipt;
use std::cell::RefCell;
use std::rc::Rc;

fn instance(observer: Box<dyn Observer>) -> Beggar<()> {
    Beggar::new(
        Origin::Fresh(Fresh {
            instance: InstanceId {
                node: 1,
                incarnation: 7,
            },
            retention: 0,
            bound: Some(1),
        }),
        observer,
    )
}

fn start(chunks: u64, sender: u64, receiver: u64) -> Command {
    Command::StartRecovery {
        chunks,
        sender,
        receiver,
    }
}

#[test]
fn process_given_a_no_op_observer_answers_as_given_one_that_records() {
    // Arrange
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut silent = instance(Box::new(NoOpObserver));
    let mut recorded = instance(Box::new(RecordingObserver::new(Rc::clone(&calls))));
    let commands = vec![
        start(0, 1, 2),
        start(8, 1, 2),
        start(4, 1, 3),
        Command::ShowRegistry,
    ];

    // Act
    let answers: Vec<(Receipt, Receipt)> = commands
        .into_iter()
        .map(|command| (silent.process(command.clone()), recorded.process(command)))
        .collect();

    // Assert
    for (from_silent, from_recorded) in answers {
        assert_eq!(from_silent, from_recorded);
    }
    assert_eq!(calls.borrow().len(), 4);
}
