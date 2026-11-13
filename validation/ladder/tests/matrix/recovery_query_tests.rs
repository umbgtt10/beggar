// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The exhaustive matrix tier for the queries: after every history of up to
// four commands -- starts, an adopt, sends and receives that advance and
// complete, cancels, and an adopt again once its session has finished -- at
// retention 0, 1, 2 and the top of u64, every query: ListRecoveries, and
// ShowRecovery and ListMissingChunks of this instance's first and second
// names, the peer's session, a name never minted and a peer's never adopted;
// from a fresh instance and from one already keeping two peer sessions, the
// later name finished earlier, at retention 2 and the top of u64; then again
// from the checkpoint of the same state, the crate's receipts and resulting
// state against the specification's model's, in its encoding. The histories
// reach a name open and
// retained at once, one retained twice, and a retention exceeded and kept.

use crate::support::model_side::ModelSide;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_writer::SpecWriter;
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
use beggar::retained_record::RetainedRecord;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use beggar_core_validation::scenario_reason::ScenarioReason;
use serde_json::Value;
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
const RETENTIONS: [u64; 4] = [0, 1, 2, u64::MAX];
const SEASONED_RETENTIONS: [u64; 2] = [2, u64::MAX];

// The commands a history is made of: this instance's sessions started, the
// peer's adopted, each advanced to a gap and completed, and each cancelled.
fn alphabet() -> Vec<Command<ScenarioReason>> {
    vec![
        Command::StartRecovery {
            chunks: 3,
            sender: INSTANCE.node,
            receiver: PEER,
        },
        Command::AdoptRecovery {
            session_id: PEERS,
            chunks: 3,
            sender: PEER,
            receiver: INSTANCE.node,
        },
        partial(own(1), 2, 2),
        partial(own(1), 1, 3),
        partial(PEERS, 2, 2),
        partial(PEERS, 1, 3),
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

// What both implementations answered to the queries after a history, and the
// state each left.
struct Answers {
    crate_receipts: Vec<Receipt<ScenarioReason>>,
    model_receipts: Vec<Value>,
    crate_state: Checkpoint,
    model_state: Value,
}

fn answers(situation: &Situation, restored: bool) -> Answers {
    let mut model = ModelSide::new(&situation.origin);
    let mut crate_side: Beggar<ScenarioReason> =
        Beggar::new(situation.origin.clone(), Box::new(NoOpObserver));
    for command in &situation.history {
        crate_side.process(command.clone());
        model.process(command);
    }
    if restored {
        let checkpoint = shown(&mut crate_side);
        model = ModelSide::new(&Origin::Checkpoint(checkpoint.clone()));
        crate_side = Beggar::new(Origin::Checkpoint(checkpoint), Box::new(NoOpObserver));
    }
    let queries = queries();
    let crate_receipts = queries
        .iter()
        .map(|command| crate_side.process(command.clone()))
        .collect();
    let model_receipts = queries
        .iter()
        .map(|command| model.process(command))
        .collect();
    Answers {
        crate_receipts,
        model_receipts,
        crate_state: shown(&mut crate_side),
        model_state: model.checkpoint(),
    }
}

fn histories() -> Vec<Vec<Command<ScenarioReason>>> {
    let mut histories = vec![Vec::new()];
    let mut last = vec![Vec::new()];
    for _ in 0..4 {
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

fn peers(count: u64) -> SessionId {
    SessionId { count, ..PEERS }
}

// Every query of every name the histories and the seasoned origin can reach,
// and two they never do.
fn queries() -> Vec<Command<ScenarioReason>> {
    let names = [own(1), own(2), PEERS, peers(2), peers(3), own(9), peers(9)];
    let mut queries = vec![Command::ListRecoveries];
    for session_id in names {
        queries.push(Command::ShowRecovery { session_id });
        queries.push(Command::ListMissingChunks { session_id });
    }
    queries
}

// The matrix row a query's output answers for.
fn row(output: &Output<ScenarioReason>) -> String {
    let row = match output {
        Output::RecoveryNotStarted => "query-not-started",
        Output::RecoveryShown { .. } => "recovery-shown",
        Output::MissingChunksListed { .. } => "missing-chunks-listed",
        Output::RecoveriesListed { .. } => "recoveries-listed",
        other => panic!("a query answered {other:?}"),
    };
    String::from(row)
}

// An instance already keeping two of the peer's sessions, the later name
// finished earlier, as a checkpoint at the retention given: the peer's third
// adopted, a chunk received and cancelled at receipts 1 to 3, its second
// adopted and completed at 4 and 5, and the registry shown at 6.
fn seasoned(retention: u64) -> Origin {
    let kept = |count: u64, held: u64, finished: u64| {
        let mut chunk_set = ChunkSet::new();
        chunk_set.add(1, held);
        RetainedRecord {
            record: SessionRecord {
                session_id: peers(count),
                chunks: 3,
                sender: PEER,
                receiver: INSTANCE.node,
                chunk_set,
            },
            finished,
        }
    };
    Origin::Checkpoint(Checkpoint {
        instance: INSTANCE,
        retention,
        bound: None,
        open: Vec::new(),
        retained: vec![kept(2, 3, 5), kept(3, 1, 3)],
        name_count: 0,
        next_receipt: 7,
    })
}

fn shown(beggar: &mut Beggar<ScenarioReason>) -> Checkpoint {
    state(beggar.process(Command::ShowRegistry))
}

// One situation: where the instance comes from and a history, the queries to
// follow. A history comes from a fresh instance at each of RETENTIONS in
// turn, then from the seasoned one at each of SEASONED_RETENTIONS.
struct Situation {
    origin: Origin,
    history: Vec<Command<ScenarioReason>>,
}

fn situations() -> Vec<Situation> {
    let mut situations = Vec::new();
    for history in histories() {
        let fresh = RETENTIONS.map(|retention| {
            Origin::Fresh(Fresh {
                instance: INSTANCE,
                retention,
                bound: None,
            })
        });
        for origin in fresh.into_iter().chain(SEASONED_RETENTIONS.map(seasoned)) {
            situations.push(Situation {
                origin,
                history: history.clone(),
            });
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
fn process_of_every_query_after_every_short_history_answers_as_the_specification_s_model() {
    // Arrange
    let situations = situations();

    // Act
    let disagreements: Vec<String> = situations
        .iter()
        .flat_map(|situation| [(situation, false), (situation, true)])
        .filter_map(|(situation, restored)| {
            let answers = answers(situation, restored);
            let agree = answers
                .crate_receipts
                .iter()
                .map(SpecWriter::receipt)
                .collect::<Vec<Value>>()
                == answers.model_receipts
                && SpecWriter::checkpoint(&answers.crate_state) == answers.model_state;
            (!agree).then(|| {
                format!(
                    "{:?} from {:?}, restored {restored}: crate {:?}, model {:?}",
                    situation.history,
                    situation.origin,
                    answers.crate_receipts,
                    answers.model_receipts
                )
            })
        })
        .collect();

    // Assert
    assert!(
        situations.len() > 28_000,
        "only {} situations",
        situations.len()
    );
    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

#[test]
fn situations_reach_a_name_open_and_retained_one_retained_twice_and_retentions_exceeded() {
    // Arrange
    let situations = situations();

    // Act
    let states: Vec<Checkpoint> = situations
        .iter()
        .map(|situation| answers(situation, false).crate_state)
        .collect();

    // Assert
    let open_and_retained = states.iter().any(|state| {
        state.open.iter().any(|record| {
            state
                .retained
                .iter()
                .any(|kept| kept.record.session_id == record.session_id)
        })
    });
    let retained_twice = states.iter().any(|state| {
        state.retained.windows(2).any(|pair| {
            pair[0].record.session_id == pair[1].record.session_id
                && pair[0].finished < pair[1].finished
        })
    });
    let alike = RETENTIONS.len() + SEASONED_RETENTIONS.len();
    let let_go = states.chunks(alike).any(|alike| {
        alike[1].retained.len() == 1
            && alike[3].retained.len() == 2
            && alike[4].retained.len() == 2
            && alike[5].retained.len() == 4
    });
    let bounded = states
        .iter()
        .all(|state| state.retained.len() as u64 <= state.retention);
    assert!(
        open_and_retained,
        "no history leaves a name open and retained"
    );
    assert!(retained_twice, "no history retains a name twice");
    assert!(
        let_go,
        "no history finishes more than retentions 1 and 2 keep"
    );
    assert!(bounded, "a retention is exceeded");
}

#[test]
fn situations_reach_every_row_the_matrix_has_for_a_query() {
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
                .any(|command| {
                    ["ListRecoveries", "ShowRecovery", "ListMissingChunks"]
                        .iter()
                        .any(|query| command == query)
                })
        })
        .map(|row| String::from(row["id"].as_str().expect("a row's id")))
        .collect();

    // Act
    let reached: BTreeSet<String> = situations()
        .iter()
        .flat_map(|situation| answers(situation, false).crate_receipts)
        .map(|receipt| row(&receipt.output))
        .collect();

    // Assert
    assert_eq!(expected.len(), 4);
    assert_eq!(reached, expected);
}
