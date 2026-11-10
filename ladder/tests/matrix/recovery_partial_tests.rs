// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The exhaustive matrix tier for partials, sent and received: every partial in
// every small situation -- a session of up to 3 chunks this node started and
// sends, or adopted from a peer and receives, holding every chunk set it can
// hold while open, alone or beside a second session; a send and a receive,
// each naming it, the second session or a name never minted, with every run
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
use beggar_core_validation::scenario_reason::ScenarioReason;
use std::collections::BTreeSet;

const BESIDE: SessionId = SessionId {
    node: 3,
    incarnation: 1,
    count: 1,
};
const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};
const NEVER_MINTED: SessionId = SessionId {
    node: 1,
    incarnation: 7,
    count: 9,
};
const NODES: [u64; 3] = [1, 2, 3];
const PEER: u64 = 2;
const PEERS_SESSION: SessionId = SessionId {
    node: PEER,
    incarnation: 5,
    count: 1,
};
const STARTED: SessionId = SessionId {
    node: 1,
    incarnation: 7,
    count: 1,
};

// What both implementations answered in a situation, and the state each left.
struct Answers {
    crate_receipt: Receipt<ScenarioReason>,
    model_receipt: Receipt<ScenarioReason>,
    crate_state: Checkpoint,
    model_state: Checkpoint,
}

fn answers(situation: &Situation, restored: bool) -> Answers {
    let fresh = Fresh {
        instance: INSTANCE,
        retention: 0,
        bound: None,
    };
    let mut crate_side: Beggar<ScenarioReason> =
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

// Every chunk set a session can hold while open: every subset of its chunks
// but the full one, which would have completed it.
fn held_sets(chunks: u64) -> Vec<Vec<u64>> {
    (0..(1u64 << chunks) - 1)
        .map(|mask| {
            (1..=chunks)
                .filter(|id| mask & (1 << (id - 1)) != 0)
                .collect()
        })
        .collect()
}

// A partial of either kind, by whether it is a receive.
fn partial(
    receiving: bool,
    session_id: SessionId,
    run: (u64, u64),
    sender: u64,
    receiver: u64,
) -> Command<ScenarioReason> {
    let (first, last) = run;
    if receiving {
        Command::ReceiveRecoveryPartial {
            session_id,
            first,
            last,
            sender,
            receiver,
        }
    } else {
        Command::SendRecoveryPartial {
            session_id,
            first,
            last,
            sender,
            receiver,
        }
    }
}

// The matrix row an output answers for.
fn row(output: &Output<ScenarioReason>) -> String {
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

// The rows the matrix gives a command.
fn rows_for(command: &str) -> BTreeSet<String> {
    SpecFiles::matrix()["rows"]
        .as_array()
        .expect("the matrix's rows")
        .iter()
        .filter(|row| {
            row["commands"]
                .as_array()
                .expect("a row's commands")
                .iter()
                .any(|listed| listed == command)
        })
        .map(|row| String::from(row["id"].as_str().expect("a row's id")))
        .collect()
}

// The calls that bring an instance to a situation: the session started and
// sent, or adopted and received, the second one adopted beside it, and each
// chunk it holds reported as a run of one.
fn setup(situation: &Situation) -> Vec<Command<ScenarioReason>> {
    let mut commands = if situation.sending {
        vec![Command::StartRecovery {
            chunks: situation.chunks,
            sender: INSTANCE.node,
            receiver: PEER,
        }]
    } else {
        vec![Command::AdoptRecovery {
            session_id: PEERS_SESSION,
            chunks: situation.chunks,
            sender: PEER,
            receiver: INSTANCE.node,
        }]
    };
    if situation.beside {
        commands.push(Command::AdoptRecovery {
            session_id: BESIDE,
            chunks: 2,
            sender: 3,
            receiver: INSTANCE.node,
        });
    }
    let (session_id, sender, receiver) = if situation.sending {
        (STARTED, INSTANCE.node, PEER)
    } else {
        (PEERS_SESSION, PEER, INSTANCE.node)
    };
    for id in &situation.held {
        commands.push(partial(
            !situation.sending,
            session_id,
            (*id, *id),
            sender,
            receiver,
        ));
    }
    commands
}

fn shown(beggar: &mut Beggar<ScenarioReason>) -> Checkpoint {
    state(beggar.process(Command::ShowRegistry))
}

// One situation: the session's chunks, whether this node is its sender, the
// chunks it holds, whether a second session is open beside it, and the partial.
struct Situation {
    chunks: u64,
    sending: bool,
    held: Vec<u64>,
    beside: bool,
    command: Command<ScenarioReason>,
}

fn situations() -> Vec<Situation> {
    let mut situations = Vec::new();
    for chunks in 1..=3 {
        for sending in [true, false] {
            let own = if sending { STARTED } else { PEERS_SESSION };
            for held in held_sets(chunks) {
                for beside in [false, true] {
                    let names = if beside {
                        vec![own, BESIDE, NEVER_MINTED]
                    } else {
                        vec![own, NEVER_MINTED]
                    };
                    for receiving in [false, true] {
                        for session_id in &names {
                            for first in 0..=chunks + 1 {
                                for last in 0..=chunks + 1 {
                                    for sender in NODES {
                                        for receiver in NODES {
                                            situations.push(Situation {
                                                chunks,
                                                sending,
                                                held: held.clone(),
                                                beside,
                                                command: partial(
                                                    receiving,
                                                    *session_id,
                                                    (first, last),
                                                    sender,
                                                    receiver,
                                                ),
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
    }
    situations
}

fn state(receipt: Receipt<ScenarioReason>) -> Checkpoint {
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
        situations.len() > 40_000,
        "only {} situations",
        situations.len()
    );
    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

#[test]
fn situations_reach_every_row_the_matrix_has_for_each_partial() {
    // Arrange
    let sends = rows_for("SendRecoveryPartial");
    let receives = rows_for("ReceiveRecoveryPartial");

    // Act
    let mut reached_by_sends = BTreeSet::new();
    let mut reached_by_receives = BTreeSet::new();
    for situation in situations() {
        let reached = row(&answers(&situation, false).crate_receipt.output);
        match situation.command {
            Command::ReceiveRecoveryPartial { .. } => reached_by_receives.insert(reached),
            _ => reached_by_sends.insert(reached),
        };
    }

    // Assert
    assert_eq!(sends.len(), 8);
    assert_eq!(reached_by_sends, sends);
    assert_eq!(reached_by_receives, receives);
}
