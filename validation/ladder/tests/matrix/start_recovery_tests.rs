// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The exhaustive matrix tier: every command in every situation over small
// bounds -- chunks up to 3, every pair of ends among three nodes, no bound and
// bounds up to 2, up to two sessions open -- from a fresh instance and from
// the checkpoint of the same state, the crate's receipt and resulting state
// against the naive model's.

use crate::support::naive_registry::NaiveRegistry;
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
use beggar_core_validation::scenario_reason::ScenarioReason;
use std::collections::BTreeSet;

const BOUNDS: [Option<u64>; 4] = [None, Some(0), Some(1), Some(2)];
const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};
const NODES: [u64; 3] = [1, 2, 3];

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
        bound: situation.bound,
    };
    let mut crate_side: Beggar<ScenarioReason> =
        Beggar::new(Origin::Fresh(fresh.clone()), Box::new(NoOpObserver));
    let mut model = NaiveRegistry::fresh(&fresh);
    for _ in 0..situation.opened {
        let open_one = start(1, INSTANCE.node, 2);
        crate_side.process(open_one.clone());
        model.answer(&open_one);
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

fn kind(output: &Output<ScenarioReason>) -> String {
    match output {
        Output::RecoveryStarted { .. } => String::from("started"),
        Output::RecoveryAdvanced { .. } => String::from("advanced"),
        Output::RecoveryCompleted => String::from("completed"),
        Output::RecoveryCancelled(_) => String::from("cancelled"),
        Output::RecoveryUnaffected => String::from("unaffected"),
        Output::RecoveryNotStarted => String::from("not-started"),
        Output::RecoveryConflicted { .. } => String::from("conflicted"),
        Output::RecoveryRejected(Rejection::RegistryFull { .. }) => String::from("registry-full"),
        Output::RecoveryRejected(rejection) => format!("{rejection:?}"),
        Output::RecoveriesListed { .. } => String::from("recoveries-listed"),
        Output::RecoveryShown { .. } => String::from("recovery-shown"),
        Output::MissingChunksListed { .. } => String::from("missing-chunks-listed"),
        Output::RegistryShown(_) => String::from("shown"),
    }
}

fn shown(beggar: &mut Beggar<ScenarioReason>) -> Checkpoint {
    state(beggar.process(Command::ShowRegistry))
}

// One situation: a bound, how many sessions are open, and the command.
struct Situation {
    bound: Option<u64>,
    opened: u64,
    command: Command<ScenarioReason>,
}

fn situations() -> Vec<Situation> {
    let mut situations = Vec::new();
    for bound in BOUNDS {
        for opened in 0..=bound.unwrap_or(2).min(2) {
            situations.push(Situation {
                bound,
                opened,
                command: Command::ShowRegistry,
            });
            for chunks in 0..=3 {
                for sender in NODES {
                    for receiver in NODES {
                        situations.push(Situation {
                            bound,
                            opened,
                            command: start(chunks, sender, receiver),
                        });
                    }
                }
            }
        }
    }
    situations
}

fn start(chunks: u64, sender: u64, receiver: u64) -> Command<ScenarioReason> {
    Command::StartRecovery {
        chunks,
        sender,
        receiver,
    }
}

fn state(receipt: Receipt<ScenarioReason>) -> Checkpoint {
    match receipt.output {
        Output::RegistryShown(checkpoint) => checkpoint,
        other => panic!("ShowRegistry answered {other:?}"),
    }
}

#[test]
fn process_in_every_small_situation_answers_as_the_naive_model() {
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
                    "bound {:?}, {} open, {:?}, restored {restored}: crate {:?}, model {:?}",
                    situation.bound,
                    situation.opened,
                    situation.command,
                    answers.crate_receipt,
                    answers.model_receipt
                )
            })
        })
        .collect();

    // Assert
    assert!(
        situations.len() > 300,
        "only {} situations",
        situations.len()
    );
    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

#[test]
fn situations_reach_every_output_the_matrix_has() {
    // Arrange
    let expected: BTreeSet<String> = [
        "started",
        "NoChunks",
        "SelfAddressed",
        "NotAParty",
        "registry-full",
        "shown",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    // Act
    let reached: BTreeSet<String> = situations()
        .iter()
        .map(|situation| kind(&answers(situation, false).crate_receipt.output))
        .collect();

    // Assert
    assert_eq!(reached, expected);
}
