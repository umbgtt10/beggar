// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The exhaustive matrix tier for cancels: after every history of up to three
// commands -- starts, an adopt, sends and receives that advance and complete,
// and cancels -- a cancel naming this instance's first or second name, the
// peer's session, a name never minted or a peer's never adopted, for two
// reasons, then a partial naming it or nothing; from a fresh instance and
// from the checkpoint of the same state, the crate's receipts and resulting
// state against the naive model's. A partial after an accepted cancel shows
// whether the cancel won.

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
use beggar::session_id::SessionId;
use beggar_core_validation::scenario_reason::ScenarioReason;
use std::collections::BTreeSet;

const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};
const PEER: u64 = 2;
const PEERS: SessionId = SessionId {
    node: PEER,
    incarnation: 5,
    count: 1,
};
const REASONS: [ScenarioReason; 2] = [
    ScenarioReason::Unservable,
    ScenarioReason::Stalled { held: 1 },
];

// The commands a history is made of: two starts' worth of this instance's
// sessions, the peer's session adopted, each advanced and completed, and each
// cancelled.
fn alphabet() -> Vec<Command<ScenarioReason>> {
    vec![
        Command::StartRecovery {
            chunks: 2,
            sender: INSTANCE.node,
            receiver: PEER,
        },
        Command::AdoptRecovery {
            session_id: PEERS,
            chunks: 2,
            sender: PEER,
            receiver: INSTANCE.node,
        },
        partial(own(1), 1, 1),
        partial(own(1), 2, 2),
        partial(PEERS, 1, 1),
        partial(PEERS, 1, 2),
        Command::CancelRecovery {
            session_id: own(1),
            reason: ScenarioReason::PeerFailed,
        },
        Command::CancelRecovery {
            session_id: PEERS,
            reason: ScenarioReason::Superseded,
        },
    ]
}

// What both implementations answered after a history, and the state each
// left.
struct Answers {
    crate_receipts: Vec<Receipt<ScenarioReason>>,
    model_receipts: Vec<Receipt<ScenarioReason>>,
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
    for command in &situation.history {
        crate_side.process(command.clone());
        model.answer(command);
    }
    if restored {
        let checkpoint = shown(&mut crate_side);
        model = NaiveRegistry::restored(&checkpoint);
        crate_side = Beggar::new(Origin::Checkpoint(checkpoint), Box::new(NoOpObserver));
    }
    let crate_receipts = situation
        .calls
        .iter()
        .map(|command| crate_side.process(command.clone()))
        .collect();
    let model_receipts = situation
        .calls
        .iter()
        .map(|command| model.answer(command))
        .collect();
    Answers {
        crate_receipts,
        model_receipts,
        crate_state: shown(&mut crate_side),
        model_state: state(model.answer(&Command::ShowRegistry)),
    }
}

fn histories() -> Vec<Vec<Command<ScenarioReason>>> {
    let mut histories = vec![Vec::new()];
    let mut last = vec![Vec::new()];
    for _ in 0..3 {
        last = last
            .iter()
            .flat_map(|history: &Vec<Command<ScenarioReason>>| {
                alphabet().into_iter().map(move |command| {
                    let mut longer = history.clone();
                    longer.push(command);
                    longer
                })
            })
            .collect();
        histories.extend(last.clone());
    }
    histories
}

fn own(count: u64) -> SessionId {
    SessionId {
        node: INSTANCE.node,
        incarnation: INSTANCE.incarnation,
        count,
    }
}

// A partial on the session named, sent if it is this instance's, received if
// it is the peer's.
fn partial(session_id: SessionId, first: u64, last: u64) -> Command<ScenarioReason> {
    if session_id.node == INSTANCE.node {
        Command::SendRecoveryPartial {
            session_id,
            first,
            last,
            sender: INSTANCE.node,
            receiver: PEER,
        }
    } else {
        Command::ReceiveRecoveryPartial {
            session_id,
            first,
            last,
            sender: PEER,
            receiver: INSTANCE.node,
        }
    }
}

// The matrix row a cancel's output answers for.
fn row(output: &Output<ScenarioReason>) -> String {
    let row = match output {
        Output::RecoveryCancelled(_) => "cancel-cancelled",
        Output::RecoveryUnaffected => "cancel-unaffected",
        other => panic!("a cancel answered {other:?}"),
    };
    String::from(row)
}

fn shown(beggar: &mut Beggar<ScenarioReason>) -> Checkpoint {
    state(beggar.process(Command::ShowRegistry))
}

// One situation: a history, then the calls under test -- a cancel, and the
// partial after it or nothing.
struct Situation {
    history: Vec<Command<ScenarioReason>>,
    calls: Vec<Command<ScenarioReason>>,
}

fn situations() -> Vec<Situation> {
    let names = [
        own(1),
        own(2),
        PEERS,
        own(9),
        SessionId { count: 9, ..PEERS },
    ];
    let mut situations = Vec::new();
    for history in histories() {
        for session_id in names {
            for reason in REASONS {
                let cancel = Command::CancelRecovery { session_id, reason };
                for followed in [false, true] {
                    let mut calls = vec![cancel.clone()];
                    if followed {
                        calls.push(partial(session_id, 1, 2));
                    }
                    situations.push(Situation {
                        history: history.clone(),
                        calls,
                    });
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
fn process_of_every_cancel_after_every_short_history_answers_as_the_naive_model() {
    // Arrange
    let situations = situations();

    // Act
    let disagreements: Vec<String> = situations
        .iter()
        .flat_map(|situation| [(situation, false), (situation, true)])
        .filter_map(|(situation, restored)| {
            let answers = answers(situation, restored);
            let agree = answers.crate_receipts == answers.model_receipts
                && answers.crate_state == answers.model_state;
            (!agree).then(|| {
                format!(
                    "{:?} then {:?}, restored {restored}: crate {:?}, model {:?}",
                    situation.history,
                    situation.calls,
                    answers.crate_receipts,
                    answers.model_receipts
                )
            })
        })
        .collect();

    // Assert
    assert!(
        situations.len() > 11_000,
        "only {} situations",
        situations.len()
    );
    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

#[test]
fn situations_reach_every_row_the_matrix_has_for_a_cancel() {
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
                .any(|command| command == "CancelRecovery")
        })
        .map(|row| String::from(row["id"].as_str().expect("a row's id")))
        .collect();

    // Act
    let reached: BTreeSet<String> = situations()
        .iter()
        .map(|situation| row(&answers(situation, false).crate_receipts[0].output))
        .collect();

    // Assert
    assert_eq!(expected.len(), 2);
    assert_eq!(reached, expected);
}
