// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The exhaustive matrix tier for partials: every partial in every small
// situation -- a session of up to 3 chunks with this node at either end, every
// chunk set it can hold while open, alone or beside a second session; a
// partial naming it, the second session or a name never minted, with every run
// from 0 to one past its chunks, under every pair of ends among three nodes --
// from a fresh instance and from the checkpoint of the same state, the crate's
// receipt and resulting state against the naive model's.

use crate::support::naive_registry::NaiveRegistry;
use crate::support::spec_files::SpecFiles;
use beggar::beggar::Beggar;
use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::no_op_observer::NoOpObserver;
use beggar::origin::Origin;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use std::collections::BTreeSet;

const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};
const NEVER_MINTED: u64 = 9;
const NODES: [u64; 3] = [1, 2, 3];

// What both implementations answered in a situation, and the state each left.
struct Answers {
    crate_receipt: Receipt,
    model_receipt: Receipt,
    crate_state: Checkpoint,
    model_state: Checkpoint,
}

fn answers(situation: &Situation, restored: bool) -> Answers {
    let fresh = Fresh {
        instance: INSTANCE,
        retention: 0,
        bound: None,
    };
    let mut crate_side: Beggar<()> =
        Beggar::new(Origin::Fresh(fresh.clone()), Box::new(NoOpObserver));
    let mut model = NaiveRegistry::fresh(&fresh);
    for command in setup(situation) {
        crate_side.process(command.clone());
        model.answer(&command);
    }
    if restored {
        let checkpoint = shown(&mut crate_side);
        model = NaiveRegistry::restored(&checkpoint);
        crate_side = Beggar::new(Origin::Checkpoint(checkpoint), Box::new(NoOpObserver));
    }
    let crate_receipt = crate_side.process(situation.command.clone());
    let model_receipt = model.answer(&situation.command);
    Answers {
        crate_receipt,
        model_receipt,
        crate_state: shown(&mut crate_side),
        model_state: state(model.answer(&Command::ShowRegistry)),
    }
}

// Every chunk set a session can hold while open: on its sender, every subset of
// its chunks but the full one, which would have completed it; on its receiver,
// none, as nothing in 0.3.0 adds to that end's set.
fn held_sets(chunks: u64, sending: bool) -> Vec<Vec<u64>> {
    if !sending {
        return vec![Vec::new()];
    }
    (0..(1u64 << chunks) - 1)
        .map(|mask| {
            (1..=chunks)
                .filter(|id| mask & (1 << (id - 1)) != 0)
                .collect()
        })
        .collect()
}

// The matrix row an output answers for.
fn row(output: &Output) -> String {
    let row = match output {
        Output::RecoveryRejected(Rejection::EmptyRun { .. }) => "empty-run",
        Output::RecoveryNotStarted => "partial-not-started",
        Output::RecoveryRejected(Rejection::Misattributed { .. }) => "misattributed",
        Output::RecoveryRejected(Rejection::WrongSide) => "wrong-side",
        Output::RecoveryRejected(Rejection::ChunkOutOfRange { .. }) => "chunk-out-of-range",
        Output::RecoveryAdvanced { .. } => "partial-advanced",
        Output::RecoveryCompleted => "partial-completed",
        Output::RecoveryUnaffected => "partial-unaffected",
        other => panic!("a partial answered {other:?}"),
    };
    String::from(row)
}

fn send(count: u64, first: u64, last: u64, sender: u64, receiver: u64) -> Command {
    Command::SendRecoveryPartial {
        session_id: SessionId {
            node: INSTANCE.node,
            incarnation: INSTANCE.incarnation,
            count,
        },
        first,
        last,
        sender,
        receiver,
    }
}

// The calls that bring an instance to a situation: the session started, the
// second one beside it, and each chunk it holds sent as a run of one.
fn setup(situation: &Situation) -> Vec<Command> {
    let (sender, receiver) = if situation.sending {
        (INSTANCE.node, 2)
    } else {
        (2, INSTANCE.node)
    };
    let mut commands = vec![start(situation.chunks, sender, receiver)];
    if situation.beside {
        commands.push(start(2, INSTANCE.node, 3));
    }
    for id in &situation.held {
        commands.push(send(1, *id, *id, sender, receiver));
    }
    commands
}

fn shown(beggar: &mut Beggar<()>) -> Checkpoint {
    state(beggar.process(Command::ShowRegistry))
}

// One situation: the session's chunks, whether this node is its sender, the
// chunks it holds, whether a second session is open beside it, and the partial.
struct Situation {
    chunks: u64,
    sending: bool,
    held: Vec<u64>,
    beside: bool,
    command: Command,
}

fn situations() -> Vec<Situation> {
    let mut situations = Vec::new();
    for chunks in 1..=3 {
        for sending in [true, false] {
            for held in held_sets(chunks, sending) {
                for beside in [false, true] {
                    let counts = if beside {
                        vec![1, 2, NEVER_MINTED]
                    } else {
                        vec![1, NEVER_MINTED]
                    };
                    for count in counts {
                        for first in 0..=chunks + 1 {
                            for last in 0..=chunks + 1 {
                                for sender in NODES {
                                    for receiver in NODES {
                                        situations.push(Situation {
                                            chunks,
                                            sending,
                                            held: held.clone(),
                                            beside,
                                            command: send(count, first, last, sender, receiver),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    situations
}

fn start(chunks: u64, sender: u64, receiver: u64) -> Command {
    Command::StartRecovery {
        chunks,
        sender,
        receiver,
    }
}

fn state(receipt: Receipt) -> Checkpoint {
    match receipt.output {
        Output::RegistryShown(checkpoint) => checkpoint,
        other => panic!("ShowRegistry answered {other:?}"),
    }
}

#[test]
fn process_of_every_partial_in_every_small_situation_answers_as_the_naive_model() {
    // Arrange
    let situations = situations();

    // Act
    let disagreements: Vec<String> = situations
        .iter()
        .flat_map(|situation| [(situation, false), (situation, true)])
        .filter_map(|(situation, restored)| {
            let answers = answers(situation, restored);
            let agree = answers.crate_receipt == answers.model_receipt
                && answers.crate_state == answers.model_state;
            (!agree).then(|| {
                format!(
                    "{} chunks, sending {}, held {:?}, beside {}, {:?}, restored {restored}: crate {:?}, model {:?}",
                    situation.chunks,
                    situation.sending,
                    situation.held,
                    situation.beside,
                    situation.command,
                    answers.crate_receipt,
                    answers.model_receipt
                )
            })
        })
        .collect();

    // Assert
    assert!(
        situations.len() > 12_000,
        "only {} situations",
        situations.len()
    );
    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

#[test]
fn situations_reach_every_row_the_matrix_has_for_a_partial() {
    // Arrange
    let expected: BTreeSet<String> = SpecFiles::matrix()["rows"]
        .as_array()
        .expect("the matrix's rows")
        .iter()
        .filter(|row| {
            row["commands"]
                .as_array()
                .expect("a row's commands")
                .iter()
                .any(|command| command == "SendRecoveryPartial")
        })
        .map(|row| String::from(row["id"].as_str().expect("a row's id")))
        .collect();

    // Act
    let reached: BTreeSet<String> = situations()
        .iter()
        .map(|situation| row(&answers(situation, false).crate_receipt.output))
        .collect();

    // Assert
    assert_eq!(expected.len(), 8);
    assert_eq!(reached, expected);
}
