// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The property tier: random sequences on random instances -- both ends of
// transfers, started and adopted, sent, received and cancelled, and queried --
// the crate against the naive model after every call, a well-formed partial
// after an accepted cancel never finding its session, the invariants on every
// state shown, the queries answering for the open and the finished sessions,
// receipts one more on every call, every call reported to the observer as it
// was answered, every command but a query answered alike at retention 0 and at
// the top of u64, one name finished again and again kept for each of its last
// finishes and read at the latest, a checkpoint taken at any point carrying on
// as the instance it was taken from, names never shared between instances, and
// a session completing exactly when every chunk is held, with no record after
// it.

use crate::support::naive_registry::NaiveRegistry;
use crate::support::random_commands::RandomCommands;
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
use beggar::retained_record::RetainedRecord;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use beggar_core_validation::scenario_reason::ScenarioReason;
use proptest::collection::btree_set;
use proptest::collection::vec;
use proptest::prelude::any;
use proptest::prelude::prop_assert;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::proptest;
use proptest::prelude::Just;
use proptest::prelude::ProptestConfig;
use proptest::prelude::Strategy;
use proptest::test_runner::TestCaseError;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

// A session finished: its name, the number of the receipt that finished it,
// and whether it completed rather than being cancelled.
struct Finish {
    session_id: SessionId,
    number: u64,
    completed: bool,
}

// What every state 0.6.0 can reach holds: the bound; as many names minted as
// starts answered; open exactly the sessions started or adopted and not
// finished, in the order of their names and none sharing one, this
// instance's own among them never beyond the names it minted; every record
// well formed, its chunk set within its chunks and never full while it is
// open; retained exactly the last sessions to finish, as many as the
// retention keeps, by name and then by finish, each full exactly when it
// completed; and the next receipt following the one that showed it.
fn holds(
    checkpoint: &Checkpoint,
    shown_by: u64,
    minted: u64,
    open: &BTreeSet<SessionId>,
    finished: &[Finish],
) -> Result<(), TestCaseError> {
    let names: Vec<SessionId> = checkpoint
        .open
        .iter()
        .map(|record| record.session_id)
        .collect();
    prop_assert!(checkpoint
        .bound
        .is_none_or(|bound| names.len() as u64 <= bound));
    prop_assert_eq!(checkpoint.name_count, minted);
    prop_assert_eq!(names, open.iter().copied().collect::<Vec<SessionId>>());
    prop_assert_eq!(checkpoint.next_receipt, shown_by + 1);
    let instance = checkpoint.instance;
    for record in &checkpoint.open {
        let own = record.session_id.node == instance.node
            && record.session_id.incarnation == instance.incarnation;
        prop_assert!(!own || record.session_id.count <= checkpoint.name_count);
        prop_assert!(record.chunks > 0);
        prop_assert!(record.sender != record.receiver);
        let node = checkpoint.instance.node;
        prop_assert!(record.sender == node || record.receiver == node);
        let ids = record.chunk_set.ids();
        prop_assert!(ids.iter().all(|id| (1..=record.chunks).contains(id)));
        prop_assert_eq!(record.chunk_set.count(), ids.len() as u64);
        prop_assert!(record.chunk_set.count() < record.chunks);
    }
    let kept = usize::try_from(checkpoint.retention).unwrap_or(usize::MAX);
    let mut expected: Vec<(SessionId, u64)> = finished[finished.len().saturating_sub(kept)..]
        .iter()
        .map(|finish| (finish.session_id, finish.number))
        .collect();
    expected.sort();
    let retained: Vec<(SessionId, u64)> = checkpoint
        .retained
        .iter()
        .map(|kept| (kept.record.session_id, kept.finished))
        .collect();
    prop_assert_eq!(retained, expected);
    for kept in &checkpoint.retained {
        let completed = finished
            .iter()
            .any(|finish| finish.number == kept.finished && finish.completed);
        let full = kept.record.chunk_set.count() == kept.record.chunks;
        prop_assert_eq!(full, completed);
    }
    Ok(())
}

fn names(receipts: &[Receipt<ScenarioReason>]) -> Vec<String> {
    receipts
        .iter()
        .filter_map(|receipt| match receipt.output {
            Output::RecoveryStarted { session_id } => Some(format!("{session_id:?}")),
            _ => None,
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn new_from_a_checkpoint_taken_anywhere_carries_on_as_its_instance(
        (fresh, before, after) in RandomCommands::fresh().prop_flat_map(|fresh| {
            let instance = fresh.instance;
            (
                Just(fresh),
                vec(RandomCommands::command(instance), 0..30),
                vec(RandomCommands::command(instance), 0..30),
            )
        }),
    ) {
        // Arrange
        let mut original: Beggar<ScenarioReason> = Beggar::new(Origin::Fresh(fresh), Box::new(NoOpObserver));
        for command in before {
            original.process(command);
        }
        let checkpoint = match original.process(Command::ShowRegistry).output {
            Output::RegistryShown(checkpoint) => checkpoint,
            other => panic!("ShowRegistry answered {other:?}"),
        };
        let mut restored: Beggar<ScenarioReason> =
            Beggar::new(Origin::Checkpoint(checkpoint), Box::new(NoOpObserver));

        // Act
        let both: Vec<(Receipt<ScenarioReason>, Receipt<ScenarioReason>)> = after
            .into_iter()
            .map(|command| (original.process(command.clone()), restored.process(command)))
            .collect();

        // Assert
        for (from_original, from_restored) in both {
            prop_assert_eq!(from_original, from_restored);
        }
    }

    #[test]
    fn process_of_any_sequence_answers_as_the_naive_model_and_keeps_the_invariants(
        (fresh, commands) in RandomCommands::fresh_and_commands(60),
    ) {
        // Arrange
        let mut beggar: Beggar<ScenarioReason> =
            Beggar::new(Origin::Fresh(fresh.clone()), Box::new(NoOpObserver));
        let mut model = NaiveRegistry::fresh(&fresh);

        // Act
        let answers: Vec<(Command<ScenarioReason>, Receipt<ScenarioReason>, Receipt<ScenarioReason>)> = commands
            .into_iter()
            .chain([Command::ShowRegistry])
            .map(|command| {
                let from_crate = beggar.process(command.clone());
                let from_model = model.answer(&command);
                (command, from_crate, from_model)
            })
            .collect();

        // Assert
        let mut minted = 0;
        let mut open: BTreeSet<SessionId> = BTreeSet::new();
        let mut cancelled: BTreeSet<SessionId> = BTreeSet::new();
        let mut finished: Vec<Finish> = Vec::new();
        for (index, (command, from_crate, from_model)) in answers.iter().enumerate() {
            prop_assert_eq!(from_crate, from_model);
            prop_assert_eq!(from_crate.number, index as u64 + 1);
            match (&from_crate.output, command) {
                (Output::RecoveryStarted { session_id }, Command::StartRecovery { .. }) => {
                    minted += 1;
                    open.insert(*session_id);
                }
                (Output::RecoveryStarted { session_id }, _) => {
                    open.insert(*session_id);
                    cancelled.remove(session_id);
                }
                (Output::RecoveryCancelled(_), Command::CancelRecovery { session_id, .. }) => {
                    open.remove(session_id);
                    cancelled.insert(*session_id);
                    finished.push(Finish { session_id: *session_id, number: from_crate.number, completed: false });
                }
                (
                    Output::RecoveryCompleted,
                    Command::SendRecoveryPartial { session_id, .. }
                    | Command::ReceiveRecoveryPartial { session_id, .. },
                ) => {
                    open.remove(session_id);
                    finished.push(Finish { session_id: *session_id, number: from_crate.number, completed: true });
                }
                (Output::RecoveriesListed { sessions }, _) => {
                    prop_assert_eq!(sessions, &open.iter().copied().collect::<Vec<SessionId>>());
                }
                (Output::RecoveryShown { record, finished: None }, _) => {
                    prop_assert!(open.contains(&record.session_id));
                }
                (Output::RecoveryShown { record, finished: Some(number) }, _) => {
                    prop_assert!(!open.contains(&record.session_id));
                    prop_assert!(finished
                        .iter()
                        .any(|finish| finish.session_id == record.session_id && finish.number == *number));
                }
                (Output::RegistryShown(checkpoint), _) => {
                    holds(checkpoint, from_crate.number, minted, &open, &finished)?;
                }
                _ => {}
            }
            if let Command::SendRecoveryPartial {
                session_id,
                first,
                last,
                ..
            }
            | Command::ReceiveRecoveryPartial {
                session_id,
                first,
                last,
                ..
            } = command
            {
                if first <= last && cancelled.contains(session_id) {
                    prop_assert_eq!(&from_crate.output, &Output::RecoveryNotStarted);
                }
            }
        }
    }

    #[test]
    fn process_at_the_highest_retention_answers_all_but_queries_as_at_retention_0(
        (fresh, commands) in RandomCommands::fresh_and_commands(60),
    ) {
        // Arrange
        let mut keeping_none: Beggar<ScenarioReason> = Beggar::new(
            Origin::Fresh(Fresh { retention: 0, ..fresh.clone() }),
            Box::new(NoOpObserver),
        );
        let mut keeping_all: Beggar<ScenarioReason> = Beggar::new(
            Origin::Fresh(Fresh { retention: u64::MAX, ..fresh }),
            Box::new(NoOpObserver),
        );

        // Act
        let answers: Vec<(Command<ScenarioReason>, Receipt<ScenarioReason>, Receipt<ScenarioReason>)> = commands
            .into_iter()
            .chain([Command::ShowRegistry])
            .map(|command| {
                let none = keeping_none.process(command.clone());
                let all = keeping_all.process(command.clone());
                (command, none, all)
            })
            .collect();

        // Assert
        for (command, none, all) in answers {
            prop_assert_eq!(none.number, all.number);
            match (command, none.output, all.output) {
                (
                    Command::ShowRecovery { .. } | Command::ListMissingChunks { .. },
                    none,
                    all,
                ) if none != all => {
                    prop_assert_eq!(none, Output::RecoveryNotStarted);
                    let of_a_finished = matches!(
                        all,
                        Output::RecoveryShown { finished: Some(_), .. } | Output::MissingChunksListed { .. }
                    );
                    prop_assert!(of_a_finished);
                }
                (Command::ShowRegistry, Output::RegistryShown(none), Output::RegistryShown(all)) => {
                    prop_assert!(none.retained.is_empty());
                    prop_assert_eq!(none, Checkpoint { retention: 0, retained: Vec::new(), ..all });
                }
                (_, none, all) => prop_assert_eq!(none, all),
            }
        }
    }

    #[test]
    fn process_of_any_sequence_reports_every_call_to_the_observer_as_answered(
        (fresh, commands) in RandomCommands::fresh_and_commands(60),
    ) {
        // Arrange
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut beggar: Beggar<ScenarioReason> = Beggar::new(
            Origin::Fresh(fresh),
            Box::new(RecordingObserver::new(Rc::clone(&calls))),
        );

        // Act
        let answered: Vec<(Command<ScenarioReason>, Receipt<ScenarioReason>)> = commands
            .into_iter()
            .map(|command| (command.clone(), beggar.process(command)))
            .collect();

        // Assert
        prop_assert_eq!(&*calls.borrow(), &answered);
    }

    #[test]
    fn process_of_one_name_adopted_and_finished_again_and_again_keeps_its_last_finishes_and_reads_the_latest(
        retention in 0u64..=4,
        rounds in vec((1u64..=4, vec((1u64..=4, 0u64..=3), 0..6)), 1..7),
    ) {
        // Arrange
        let instance = InstanceId { node: 1, incarnation: 7 };
        let peers = SessionId { node: 2, incarnation: 5, count: 1 };
        let mut beggar: Beggar<ScenarioReason> = Beggar::new(
            Origin::Fresh(Fresh { instance, retention, bound: None }),
            Box::new(NoOpObserver),
        );
        let rounds: Vec<(u64, Vec<(u64, u64)>)> = rounds
            .into_iter()
            .map(|(chunks, runs)| {
                let runs = runs
                    .into_iter()
                    .map(|(first, span)| {
                        let first = (first - 1) % chunks + 1;
                        (first, (first + span).min(chunks))
                    })
                    .collect();
                (chunks, runs)
            })
            .collect();
        let commands: Vec<Command<ScenarioReason>> = rounds
            .iter()
            .flat_map(|(chunks, runs)| {
                let adopt = Command::AdoptRecovery { session_id: peers, chunks: *chunks, sender: 2, receiver: 1 };
                let receives = runs.iter().map(move |(first, last)| Command::ReceiveRecoveryPartial {
                    session_id: peers,
                    first: *first,
                    last: *last,
                    sender: 2,
                    receiver: 1,
                });
                let cancel = Command::CancelRecovery { session_id: peers, reason: ScenarioReason::PeerFailed };
                [adopt].into_iter().chain(receives).chain([cancel])
            })
            .collect();

        // Act
        let finishes: Vec<u64> = commands
            .into_iter()
            .map(|command| beggar.process(command))
            .filter(|receipt| matches!(receipt.output, Output::RecoveryCompleted | Output::RecoveryCancelled(_)))
            .map(|receipt| receipt.number)
            .collect();
        let shown = beggar.process(Command::ShowRecovery { session_id: peers }).output;
        let retained = match beggar.process(Command::ShowRegistry).output {
            Output::RegistryShown(checkpoint) => checkpoint.retained,
            other => panic!("ShowRegistry answered {other:?}"),
        };

        // Assert
        prop_assert_eq!(finishes.len(), rounds.len());
        let kept: Vec<RetainedRecord> = rounds
            .iter()
            .zip(&finishes)
            .map(|((chunks, runs), finished)| {
                let mut chunk_set = ChunkSet::new();
                for (first, last) in runs {
                    chunk_set.add(*first, *last);
                }
                RetainedRecord {
                    record: SessionRecord { session_id: peers, chunks: *chunks, sender: 2, receiver: 1, chunk_set },
                    finished: *finished,
                }
            })
            .collect();
        let last = usize::try_from(retention).unwrap_or(usize::MAX).min(kept.len());
        prop_assert_eq!(&retained, &kept[kept.len() - last..].to_vec());
        let latest = kept.last().filter(|_| retention > 0).map_or(Output::RecoveryNotStarted, |latest| {
            Output::RecoveryShown { record: latest.record.clone(), finished: Some(latest.finished) }
        });
        prop_assert_eq!(shown, latest);
    }

    #[test]
    fn process_of_runs_on_one_session_completes_exactly_when_every_chunk_is_held_and_keeps_no_record_after(
        receiving in any::<bool>(),
        chunks in 1u64..=24,
        runs in vec((1u64..=24, 0u64..=6), 1..40),
    ) {
        // Arrange
        let instance = InstanceId { node: 1, incarnation: 7 };
        let mut beggar: Beggar<ScenarioReason> = Beggar::new(
            Origin::Fresh(Fresh { instance, retention: 0, bound: None }),
            Box::new(NoOpObserver),
        );
        let peers = SessionId { node: 2, incarnation: 5, count: 1 };
        let (session_id, sender, receiver) = if receiving {
            beggar.process(Command::AdoptRecovery { session_id: peers, chunks, sender: 2, receiver: 1 });
            (peers, 2, 1)
        } else {
            beggar.process(Command::StartRecovery { chunks, sender: 1, receiver: 2 });
            (SessionId { node: 1, incarnation: 7, count: 1 }, 1, 2)
        };
        let runs: Vec<(u64, u64)> = runs
            .into_iter()
            .map(|(first, span)| {
                let first = (first - 1) % chunks + 1;
                (first, (first + span).min(chunks))
            })
            .collect();

        // Act
        let outputs: Vec<Output<ScenarioReason>> = runs
            .iter()
            .map(|(first, last)| {
                let (first, last) = (*first, *last);
                let partial = if receiving {
                    Command::ReceiveRecoveryPartial { session_id, first, last, sender, receiver }
                } else {
                    Command::SendRecoveryPartial { session_id, first, last, sender, receiver }
                };
                beggar.process(partial).output
            })
            .collect();
        let open_after = match beggar.process(Command::ShowRegistry).output {
            Output::RegistryShown(checkpoint) => checkpoint.open.len(),
            other => panic!("ShowRegistry answered {other:?}"),
        };

        // Assert
        let mut held: BTreeSet<u64> = BTreeSet::new();
        let mut completed = false;
        for ((first, last), output) in runs.iter().zip(&outputs) {
            let before = held.len();
            held.extend(*first..=*last);
            let expected = if completed {
                Output::RecoveryNotStarted
            } else if held.len() == before {
                Output::RecoveryUnaffected
            } else if held.len() as u64 == chunks {
                completed = true;
                Output::RecoveryCompleted
            } else {
                Output::RecoveryAdvanced { held: held.len() as u64, chunks }
            };
            prop_assert_eq!(output, &expected);
        }
        prop_assert_eq!(open_after, usize::from(!completed));
    }

    #[test]
    fn process_on_distinct_instances_never_mints_one_name_twice(
        instances in btree_set((1u64..=3, 0u64..=3), 2..6),
        starts in 1usize..6,
    ) {
        // Arrange
        let mut registries: Vec<Beggar<ScenarioReason>> = instances
            .iter()
            .map(|(node, incarnation)| {
                Beggar::new(
                    Origin::Fresh(Fresh {
                        instance: InstanceId { node: *node, incarnation: *incarnation },
                        retention: 0,
                        bound: None,
                    }),
                    Box::new(NoOpObserver),
                )
            })
            .collect();

        // Act
        let receipts: Vec<Receipt<ScenarioReason>> = registries
            .iter_mut()
            .zip(instances.iter())
            .flat_map(|(registry, (node, _))| {
                let peer = node % 3 + 1;
                (0..starts)
                    .map(|_| {
                        registry.process(Command::StartRecovery {
                            chunks: 1,
                            sender: *node,
                            receiver: peer,
                        })
                    })
                    .collect::<Vec<Receipt<ScenarioReason>>>()
            })
            .collect();

        // Assert
        let minted = names(&receipts);
        let distinct: BTreeSet<&String> = minted.iter().collect();
        prop_assert_eq!(minted.len(), instances.len() * starts);
        prop_assert_eq!(distinct.len(), minted.len());
    }
}
