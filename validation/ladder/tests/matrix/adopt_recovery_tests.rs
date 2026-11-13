// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The exhaustive matrix tier for adopts: every adopt in every small situation
// -- no bound and bounds up to 2; up to two sessions adopted from a peer, and
// one this node started, open or not; an adopt naming an open peer's session,
// a peer's not open, this instance's own started and next names, its node's
// previous incarnation's, and another node's; chunks up to 3, every pair of
// ends among three nodes -- from a fresh instance and from the checkpoint of
// the same state, the crate's receipt and resulting state against the
// specification's model's, in its encoding.

use crate::support::model_side::ModelSide;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_writer::SpecWriter;
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
use serde_json::Value;
use std::collections::BTreeSet;

const BOUNDS: [Option<u64>; 4] = [None, Some(0), Some(1), Some(2)];
const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};
const NAMES: [SessionId; 7] = [
    peers(1),
    peers(2),
    peers(3),
    SessionId {
        node: 1,
        incarnation: 7,
        count: 1,
    },
    SessionId {
        node: 1,
        incarnation: 7,
        count: 2,
    },
    SessionId {
        node: 1,
        incarnation: 6,
        count: 1,
    },
    SessionId {
        node: 3,
        incarnation: 1,
        count: 1,
    },
];
const NODES: [u64; 3] = [1, 2, 3];
const PEER: u64 = 2;

fn adopt(
    session_id: SessionId,
    chunks: u64,
    sender: u64,
    receiver: u64,
) -> Command<ScenarioReason> {
    Command::AdoptRecovery {
        session_id,
        chunks,
        sender,
        receiver,
    }
}

// What both implementations answered in a situation, and the state each left.
struct Answers {
    crate_receipt: Receipt<ScenarioReason>,
    model_receipt: Value,
    crate_state: Checkpoint,
    model_state: Value,
}

fn answers(situation: &Situation, restored: bool) -> Answers {
    let fresh = Fresh {
        instance: INSTANCE,
        retention: 0,
        bound: situation.bound,
    };
    let mut crate_side: Beggar<ScenarioReason> =
        Beggar::new(Origin::Fresh(fresh.clone()), Box::new(NoOpObserver));
    let mut model = ModelSide::new(&Origin::Fresh(fresh.clone()));
    for command in setup(situation) {
        crate_side.process(command.clone());
        model.process(&command);
    }
    if restored {
        let checkpoint = shown(&mut crate_side);
        model = ModelSide::new(&Origin::Checkpoint(checkpoint.clone()));
        crate_side = Beggar::new(Origin::Checkpoint(checkpoint), Box::new(NoOpObserver));
    }
    let crate_receipt = crate_side.process(situation.command.clone());
    let model_receipt = model.process(&situation.command);
    Answers {
        crate_receipt,
        model_receipt,
        crate_state: shown(&mut crate_side),
        model_state: model.checkpoint(),
    }
}

const fn peers(count: u64) -> SessionId {
    SessionId {
        node: PEER,
        incarnation: 5,
        count,
    }
}

// The matrix row an adopt's output answers for.
fn row(output: &Output<ScenarioReason>) -> String {
    let row = match output {
        Output::RecoveryRejected(Rejection::NoChunks) => "no-chunks",
        Output::RecoveryRejected(Rejection::SelfAddressed) => "self-addressed",
        Output::RecoveryRejected(Rejection::NotAParty) => "not-a-party",
        Output::RecoveryRejected(Rejection::OwnSession) => "own-session",
        Output::RecoveryUnaffected => "adopt-unaffected",
        Output::RecoveryConflicted { .. } => "adopt-conflicted",
        Output::RecoveryRejected(Rejection::RegistryFull { .. }) => "adopt-registry-full",
        Output::RecoveryStarted { .. } => "adopt-started",
        other => panic!("an adopt answered {other:?}"),
    };
    String::from(row)
}

// The calls that bring an instance to a situation: the peer's sessions
// adopted, two chunks each, and this node's own started beside them.
fn setup(situation: &Situation) -> Vec<Command<ScenarioReason>> {
    let mut commands: Vec<Command<ScenarioReason>> = (1..=situation.adopted)
        .map(|count| adopt(peers(count), 2, PEER, INSTANCE.node))
        .collect();
    if situation.started {
        commands.push(Command::StartRecovery {
            chunks: 2,
            sender: INSTANCE.node,
            receiver: 3,
        });
    }
    commands
}

fn shown(beggar: &mut Beggar<ScenarioReason>) -> Checkpoint {
    state(beggar.process(Command::ShowRegistry))
}

// One situation: a bound, how many of the peer's sessions are adopted, whether
// this node's own started session is open, and the adopt.
struct Situation {
    bound: Option<u64>,
    adopted: u64,
    started: bool,
    command: Command<ScenarioReason>,
}

fn situations() -> Vec<Situation> {
    let mut situations = Vec::new();
    for bound in BOUNDS {
        for adopted in 0..=2 {
            for started in [false, true] {
                if bound.is_some_and(|bound| adopted + u64::from(started) > bound) {
                    continue;
                }
                for session_id in NAMES {
                    for chunks in 0..=3 {
                        for sender in NODES {
                            for receiver in NODES {
                                situations.push(Situation {
                                    bound,
                                    adopted,
                                    started,
                                    command: adopt(session_id, chunks, sender, receiver),
                                });
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
fn process_of_every_adopt_in_every_small_situation_answers_as_the_specification_s_model() {
    // Arrange
    let situations = situations();

    // Act
    let disagreements: Vec<String> = situations
        .iter()
        .flat_map(|situation| [(situation, false), (situation, true)])
        .filter_map(|(situation, restored)| {
            let answers = answers(situation, restored);
            let agree = SpecWriter::receipt(&answers.crate_receipt) == answers.model_receipt
                && SpecWriter::checkpoint(&answers.crate_state) == answers.model_state;
            (!agree).then(|| {
                format!(
                    "bound {:?}, {} adopted, started {}, {:?}, restored {restored}: crate {:?}, model {:?}",
                    situation.bound,
                    situation.adopted,
                    situation.started,
                    situation.command,
                    answers.crate_receipt,
                    answers.model_receipt
                )
            })
        })
        .collect();

    // Assert
    assert!(
        situations.len() > 3_500,
        "only {} situations",
        situations.len()
    );
    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

#[test]
fn situations_reach_every_row_the_matrix_has_for_an_adopt() {
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
                .any(|command| command == "AdoptRecovery")
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
