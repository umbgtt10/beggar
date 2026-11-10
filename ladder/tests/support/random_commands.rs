// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::session_id::SessionId;
use proptest::collection::vec;
use proptest::option::of;
use proptest::prelude::any;
use proptest::prelude::prop_oneof;
use proptest::prelude::Just;
use proptest::prelude::Strategy;

// The random instances and commands the property and replay tiers drive the
// crate with: starts and partials mostly between this node and its usual peer,
// so partials often find their session, otherwise between any two of three
// nodes; a partial naming one of the instance's first five names, its run
// mostly well formed.
pub struct RandomCommands;

impl RandomCommands {
    pub fn command(instance: InstanceId) -> impl Strategy<Value = Command> {
        let run = prop_oneof![
            4 => (0u64..=7, 0u64..=3).prop_map(|(first, span)| (first, first + span)),
            1 => (0u64..=7, 0u64..=7),
        ];
        prop_oneof![
            3 => (0u64..=6, Self::ends(instance.node)).prop_map(|(chunks, (sender, receiver))| {
                Command::StartRecovery {
                    chunks,
                    sender,
                    receiver,
                }
            }),
            4 => (1u64..=5, run, Self::ends(instance.node)).prop_map(
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

    pub fn fresh() -> impl Strategy<Value = Fresh> {
        (1u64..=3, any::<u64>(), 0u64..=8, of(0u64..=4)).prop_map(
            |(node, incarnation, retention, bound)| Fresh {
                instance: InstanceId { node, incarnation },
                retention,
                bound,
            },
        )
    }

    // A random instance and a random sequence of its commands.
    pub fn fresh_and_commands(longest: usize) -> impl Strategy<Value = (Fresh, Vec<Command>)> {
        Self::fresh().prop_flat_map(move |fresh| {
            let instance = fresh.instance;
            (Just(fresh), vec(Self::command(instance), 0..longest))
        })
    }

    // A command's sender and receiver: this node and its usual peer, three
    // times in four, otherwise any two of three nodes.
    fn ends(node: u64) -> impl Strategy<Value = (u64, u64)> {
        prop_oneof![
            3 => Just((node, node % 3 + 1)),
            1 => (1u64..=3, 1u64..=3),
        ]
    }
}
