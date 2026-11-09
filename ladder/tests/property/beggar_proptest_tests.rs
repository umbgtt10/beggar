// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The property tier: random sequences on random instances, the crate against
// the naive model after every call, the invariants on every state shown,
// receipts one more on every call, a checkpoint taken at any point carrying on
// as the instance it was taken from, and names never shared between instances.

use crate::support::naive_registry::NaiveRegistry;
use beggar::beggar::Beggar;
use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::origin::Origin;
use beggar::output::Output;
use beggar::receipt::Receipt;
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
use std::collections::BTreeSet;

fn command() -> impl Strategy<Value = Command> {
    prop_oneof![
        4 => (0u64..=4, 1u64..=4, 1u64..=4).prop_map(|(chunks, sender, receiver)| {
            Command::StartRecovery {
                chunks,
                sender,
                receiver,
            }
        }),
        1 => Just(Command::ShowRegistry),
    ]
}

fn fresh() -> impl Strategy<Value = Fresh> {
    (1u64..=4, any::<u64>(), 0u64..=8, of(0u64..=4)).prop_map(
        |(node, incarnation, retention, bound)| Fresh {
            instance: InstanceId { node, incarnation },
            retention,
            bound,
        },
    )
}

// What every state 0.1.0 can reach holds: the bound, the names minted in turn
// and all still open, every record well formed, and the next receipt following
// the one that showed it.
fn holds(checkpoint: &Checkpoint, shown_by: u64) -> Result<(), TestCaseError> {
    let open = checkpoint.open.len() as u64;
    prop_assert!(checkpoint.bound.is_none_or(|bound| open <= bound));
    prop_assert_eq!(open, checkpoint.name_count);
    prop_assert_eq!(checkpoint.next_receipt, shown_by + 1);
    for (index, record) in checkpoint.open.iter().enumerate() {
        prop_assert_eq!(record.session_id.count, index as u64 + 1);
        prop_assert_eq!(record.session_id.node, checkpoint.instance.node);
        prop_assert_eq!(
            record.session_id.incarnation,
            checkpoint.instance.incarnation
        );
        prop_assert!(record.chunks > 0);
        prop_assert!(record.sender != record.receiver);
        let node = checkpoint.instance.node;
        prop_assert!(record.sender == node || record.receiver == node);
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
        fresh in fresh(),
        before in vec(command(), 0..30),
        after in vec(command(), 0..30),
    ) {
        // Arrange
        let mut original: Beggar<()> = Beggar::new(Origin::Fresh(fresh));
        for command in before {
            original.process(command);
        }
        let checkpoint = match original.process(Command::ShowRegistry).output {
            Output::RegistryShown(checkpoint) => checkpoint,
            other => panic!("ShowRegistry answered {other:?}"),
        };
        let mut restored: Beggar<()> = Beggar::new(Origin::Checkpoint(checkpoint));

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
        fresh in fresh(),
        commands in vec(command(), 0..60),
    ) {
        // Arrange
        let mut beggar: Beggar<()> = Beggar::new(Origin::Fresh(fresh.clone()));
        let mut model = NaiveRegistry::fresh(&fresh);

        // Act
        let answers: Vec<(Receipt, Receipt)> = commands
            .iter()
            .chain([Command::ShowRegistry].iter())
            .map(|command| (beggar.process(command.clone()), model.answer(command)))
            .collect();

        // Assert
        for (index, (from_crate, from_model)) in answers.iter().enumerate() {
            prop_assert_eq!(from_crate, from_model);
            prop_assert_eq!(from_crate.number, index as u64 + 1);
            if let Output::RegistryShown(checkpoint) = &from_crate.output {
                holds(checkpoint, from_crate.number)?;
            }
        }
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
                Beggar::new(Origin::Fresh(Fresh {
                    instance: InstanceId { node: *node, incarnation: *incarnation },
                    retention: 0,
                    bound: None,
                }))
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
