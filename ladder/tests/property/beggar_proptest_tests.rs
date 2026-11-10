// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The property tier: random sequences on random instances, the crate against
// the naive model after every call, the invariants on every state shown,
// receipts one more on every call, every call reported to the observer as it
// was answered, a checkpoint taken at any point carrying on as the instance it
// was taken from, names never shared between instances, and a session
// completing exactly when every chunk is held, with no record after it.

use crate::support::naive_registry::NaiveRegistry;
use crate::support::recording_observer::RecordingObserver;
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
use proptest::collection::btree_set;
use proptest::collection::vec;
use proptest::option::of;
use proptest::prelude::any;
use proptest::prelude::prop_assert;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::prop_oneof;
use proptest::prelude::proptest;
use proptest::prelude::Just;
use proptest::prelude::ProptestConfig;
use proptest::prelude::Strategy;
use proptest::test_runner::TestCaseError;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

// A random command for an instance: starts and partials mostly between this
// node and its usual peer, so partials often find their session, otherwise
// between any two of three nodes; a partial naming one of the instance's first
// five names, its run mostly well formed.
fn command(instance: InstanceId) -> impl Strategy<Value = Command> {
    let run = prop_oneof![
        4 => (0u64..=7, 0u64..=3).prop_map(|(first, span)| (first, first + span)),
        1 => (0u64..=7, 0u64..=7),
    ];
    prop_oneof![
        3 => (0u64..=6, ends(instance.node)).prop_map(|(chunks, (sender, receiver))| {
            Command::StartRecovery {
                chunks,
                sender,
                receiver,
            }
        }),
        4 => (1u64..=5, run, ends(instance.node)).prop_map(
            move |(count, (first, last), (sender, receiver))| Command::SendRecoveryPartial {
                session_id: SessionId {
                    node: instance.node,
                    incarnation: instance.incarnation,
                    count,
                },
                first,
                last,
                sender,
                receiver,
            }
        ),
        1 => Just(Command::ShowRegistry),
    ]
}

// A command's sender and receiver: this node and its usual peer, three times
// in four, otherwise any two of three nodes.
fn ends(node: u64) -> impl Strategy<Value = (u64, u64)> {
    prop_oneof![
        3 => Just((node, node % 3 + 1)),
        1 => (1u64..=3, 1u64..=3),
    ]
}

fn fresh() -> impl Strategy<Value = Fresh> {
    (1u64..=3, any::<u64>(), 0u64..=8, of(0u64..=4)).prop_map(
        |(node, incarnation, retention, bound)| Fresh {
            instance: InstanceId { node, incarnation },
            retention,
            bound,
        },
    )
}

// A random instance and a random sequence of its commands.
fn fresh_and_commands(longest: usize) -> impl Strategy<Value = (Fresh, Vec<Command>)> {
    fresh().prop_flat_map(move |fresh| {
        let instance = fresh.instance;
        (Just(fresh), vec(command(instance), 0..longest))
    })
}

// What every state 0.3.0 can reach holds: the bound; as many names minted as
// starts answered; open exactly the sessions started and not completed, in the
// order of their names; every record well formed, its chunk set within its
// chunks and never full while it is open; and the next receipt following the
// one that showed it.
fn holds(
    checkpoint: &Checkpoint,
    shown_by: u64,
    minted: u64,
    open: &[u64],
) -> Result<(), TestCaseError> {
    let counts: Vec<u64> = checkpoint
        .open
        .iter()
        .map(|record| record.session_id.count)
        .collect();
    prop_assert!(checkpoint
        .bound
        .is_none_or(|bound| counts.len() as u64 <= bound));
    prop_assert_eq!(checkpoint.name_count, minted);
    prop_assert_eq!(counts.as_slice(), open);
    prop_assert_eq!(checkpoint.next_receipt, shown_by + 1);
    for record in &checkpoint.open {
        prop_assert_eq!(record.session_id.node, checkpoint.instance.node);
        prop_assert_eq!(
            record.session_id.incarnation,
            checkpoint.instance.incarnation
        );
        prop_assert!(record.chunks > 0);
        prop_assert!(record.sender != record.receiver);
        let node = checkpoint.instance.node;
        prop_assert!(record.sender == node || record.receiver == node);
        let ids = record.chunk_set.ids();
        prop_assert!(ids.iter().all(|id| (1..=record.chunks).contains(id)));
        prop_assert_eq!(record.chunk_set.count(), ids.len() as u64);
        prop_assert!(record.chunk_set.count() < record.chunks);
    }
    Ok(())
}

fn names(receipts: &[Receipt]) -> Vec<String> {
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
        (fresh, before, after) in fresh().prop_flat_map(|fresh| {
            let instance = fresh.instance;
            (Just(fresh), vec(command(instance), 0..30), vec(command(instance), 0..30))
        }),
    ) {
        // Arrange
        let mut original: Beggar<()> = Beggar::new(Origin::Fresh(fresh), Box::new(NoOpObserver));
        for command in before {
            original.process(command);
        }
        let checkpoint = match original.process(Command::ShowRegistry).output {
            Output::RegistryShown(checkpoint) => checkpoint,
            other => panic!("ShowRegistry answered {other:?}"),
        };
        let mut restored: Beggar<()> =
            Beggar::new(Origin::Checkpoint(checkpoint), Box::new(NoOpObserver));

        // Act
        let both: Vec<(Receipt, Receipt)> = after
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
        (fresh, commands) in fresh_and_commands(60),
    ) {
        // Arrange
        let mut beggar: Beggar<()> =
            Beggar::new(Origin::Fresh(fresh.clone()), Box::new(NoOpObserver));
        let mut model = NaiveRegistry::fresh(&fresh);

        // Act
        let answers: Vec<(Command, Receipt, Receipt)> = commands
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
        let mut open: Vec<u64> = Vec::new();
        for (index, (command, from_crate, from_model)) in answers.iter().enumerate() {
            prop_assert_eq!(from_crate, from_model);
            prop_assert_eq!(from_crate.number, index as u64 + 1);
            match (&from_crate.output, command) {
                (Output::RecoveryStarted { session_id }, _) => {
                    minted += 1;
                    open.push(session_id.count);
                }
                (Output::RecoveryCompleted, Command::SendRecoveryPartial { session_id, .. }) => {
                    open.retain(|count| *count != session_id.count);
                }
                (Output::RegistryShown(checkpoint), _) => {
                    holds(checkpoint, from_crate.number, minted, &open)?;
                }
                _ => {}
            }
        }
    }

    #[test]
    fn process_of_any_sequence_reports_every_call_to_the_observer_as_answered(
        (fresh, commands) in fresh_and_commands(60),
    ) {
        // Arrange
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut beggar: Beggar<()> = Beggar::new(
            Origin::Fresh(fresh),
            Box::new(RecordingObserver::new(Rc::clone(&calls))),
        );

        // Act
        let answered: Vec<(Command, Receipt)> = commands
            .into_iter()
            .map(|command| (command.clone(), beggar.process(command)))
            .collect();

        // Assert
        prop_assert_eq!(&*calls.borrow(), &answered);
    }

    #[test]
    fn process_of_runs_on_one_session_completes_exactly_when_every_chunk_is_held_and_keeps_no_record_after(
        chunks in 1u64..=24,
        runs in vec((1u64..=24, 0u64..=6), 1..40),
    ) {
        // Arrange
        let instance = InstanceId { node: 1, incarnation: 7 };
        let mut beggar: Beggar<()> = Beggar::new(
            Origin::Fresh(Fresh { instance, retention: 0, bound: None }),
            Box::new(NoOpObserver),
        );
        beggar.process(Command::StartRecovery { chunks, sender: 1, receiver: 2 });
        let runs: Vec<(u64, u64)> = runs
            .into_iter()
            .map(|(first, span)| {
                let first = (first - 1) % chunks + 1;
                (first, (first + span).min(chunks))
            })
            .collect();

        // Act
        let outputs: Vec<Output> = runs
            .iter()
            .map(|(first, last)| {
                beggar
                    .process(Command::SendRecoveryPartial {
                        session_id: SessionId { node: 1, incarnation: 7, count: 1 },
                        first: *first,
                        last: *last,
                        sender: 1,
                        receiver: 2,
                    })
                    .output
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
        let mut registries: Vec<Beggar<()>> = instances
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
        let receipts: Vec<Receipt> = registries
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
                    .collect::<Vec<Receipt>>()
            })
            .collect();

        // Assert
        let minted = names(&receipts);
        let distinct: BTreeSet<&String> = minted.iter().collect();
        prop_assert_eq!(minted.len(), instances.len() * starts);
        prop_assert_eq!(distinct.len(), minted.len());
    }
}
