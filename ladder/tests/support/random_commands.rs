// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::session_id::SessionId;
use beggar_core_validation::scenario_reason::ScenarioReason;
use proptest::collection::vec;
use proptest::option::of;
use proptest::prelude::any;
use proptest::prelude::prop_oneof;
use proptest::prelude::Just;
use proptest::prelude::Strategy;

const PEER_INCARNATION: u64 = 5;

// The random instances and commands the property and replay tiers drive the
// crate with: starts and sends mostly from this node to its usual peer, adopts
// and receives mostly from that peer to this node, so partials often find
// their session, otherwise between any two of three nodes; a partial naming
// one of the first five names, this instance's to send, the peer's to receive;
// an adopt naming the peer's, now and then this instance's own; a cancel of
// either, for any of the four reasons; ShowRecovery and ListMissingChunks of
// either, and ListRecoveries; runs mostly well formed.
pub struct RandomCommands;

impl RandomCommands {
    pub fn command(instance: InstanceId) -> impl Strategy<Value = Command<ScenarioReason>> {
        let node = instance.node;
        let peer = node % 3 + 1;
        prop_oneof![
            3 => (0u64..=6, Self::ends(node, peer)).prop_map(|(chunks, (sender, receiver))| {
                Command::StartRecovery {
                    chunks,
                    sender,
                    receiver,
                }
            }),
            3 => (Self::adopted_name(instance, peer), 0u64..=6, Self::ends(peer, node)).prop_map(
                |(session_id, chunks, (sender, receiver))| Command::AdoptRecovery {
                    session_id,
                    chunks,
                    sender,
                    receiver,
                }
            ),
            3 => (Self::own_name(instance), Self::run(), Self::ends(node, peer)).prop_map(
                |(session_id, (first, last), (sender, receiver))| Command::SendRecoveryPartial {
                    session_id,
                    first,
                    last,
                    sender,
                    receiver,
                }
            ),
            3 => (Self::peers_name(peer), Self::run(), Self::ends(peer, node)).prop_map(
                |(session_id, (first, last), (sender, receiver))| Command::ReceiveRecoveryPartial {
                    session_id,
                    first,
                    last,
                    sender,
                    receiver,
                }
            ),
            2 => (Self::either_name(instance, peer), Self::reason()).prop_map(
                |(session_id, reason)| Command::CancelRecovery { session_id, reason }
            ),
            1 => Just(Command::ListRecoveries),
            1 => Self::either_name(instance, peer)
                .prop_map(|session_id| Command::ShowRecovery { session_id }),
            1 => Self::either_name(instance, peer)
                .prop_map(|session_id| Command::ListMissingChunks { session_id }),
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
    pub fn fresh_and_commands(
        longest: usize,
    ) -> impl Strategy<Value = (Fresh, Vec<Command<ScenarioReason>>)> {
        Self::fresh().prop_flat_map(move |fresh| {
            let instance = fresh.instance;
            (Just(fresh), vec(Self::command(instance), 0..longest))
        })
    }

    // An adopt's name: the peer's, four times in six; this instance's own,
    // which an adopt is refused; or one from its node's previous incarnation,
    // another instance's, which is adopted.
    fn adopted_name(instance: InstanceId, peer: u64) -> impl Strategy<Value = SessionId> {
        let previous = InstanceId {
            node: instance.node,
            incarnation: instance.incarnation.wrapping_sub(1),
        };
        prop_oneof![
            4 => Self::peers_name(peer),
            1 => Self::own_name(instance),
            1 => Self::own_name(previous),
        ]
    }

    // A cancel's or a query's name: this instance's or the peer's, alike.
    fn either_name(instance: InstanceId, peer: u64) -> impl Strategy<Value = SessionId> {
        prop_oneof![Self::own_name(instance), Self::peers_name(peer)]
    }

    // A command's sender and receiver: the usual pair, three times in four,
    // otherwise any two of three nodes.
    fn ends(sender: u64, receiver: u64) -> impl Strategy<Value = (u64, u64)> {
        prop_oneof![
            3 => Just((sender, receiver)),
            1 => (1u64..=3, 1u64..=3),
        ]
    }

    fn own_name(instance: InstanceId) -> impl Strategy<Value = SessionId> {
        (1u64..=5).prop_map(move |count| SessionId {
            node: instance.node,
            incarnation: instance.incarnation,
            count,
        })
    }

    fn peers_name(peer: u64) -> impl Strategy<Value = SessionId> {
        (1u64..=5).prop_map(move |count| SessionId {
            node: peer,
            incarnation: PEER_INCARNATION,
            count,
        })
    }

    fn reason() -> impl Strategy<Value = ScenarioReason> {
        prop_oneof![
            Just(ScenarioReason::Unservable),
            Just(ScenarioReason::Superseded),
            Just(ScenarioReason::PeerFailed),
            (0u64..=6).prop_map(|held| ScenarioReason::Stalled { held }),
        ]
    }

    fn run() -> impl Strategy<Value = (u64, u64)> {
        prop_oneof![
            4 => (0u64..=7, 0u64..=3).prop_map(|(first, span)| (first, first + span)),
            1 => (0u64..=7, 0u64..=7),
        ]
    }
}
