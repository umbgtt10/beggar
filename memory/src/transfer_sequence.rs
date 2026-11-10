// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::allocation_counter::AllocationCounter;
use crate::memory_profile::MemoryProfile;
use beggar::beggar::Beggar;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::no_op_observer::NoOpObserver;
use beggar::origin::Origin;
use beggar::session_id::SessionId;
use std::array::from_fn;

const INCARNATION: u64 = 1;
const NODES: usize = 5;
const RECEIVER: u64 = 2;
const SENDER: u64 = 1;

// Recoveries one after another between two of five instances at retention 0,
// as DESIGN.md has the memory test: each a session of as many chunks as the
// order names, started by the sender, adopted by the receiver from its first
// run, every chunk sent and received a run of one in that order until both
// ends complete it; the other three take no part. The instances are kept on
// the stack, so every byte counted is beggar's, from before they exist.
pub struct TransferSequence<'a> {
    counter: &'a dyn AllocationCounter,
    order: Vec<u64>,
}

impl<'a> TransferSequence<'a> {
    pub fn new(counter: &'a dyn AllocationCounter, order: Vec<u64>) -> Self {
        Self { counter, order }
    }

    pub fn run(&self, recoveries: u64) -> MemoryProfile {
        self.counter.start();
        let mut nodes: [Beggar<()>; NODES] = from_fn(|index| {
            Beggar::new(
                Origin::Fresh(Fresh {
                    instance: InstanceId {
                        node: index as u64 + 1,
                        incarnation: INCARNATION,
                    },
                    retention: 0,
                    bound: None,
                }),
                Box::new(NoOpObserver),
            )
        });
        let skeleton = self.counter.live();
        let mut held_at_worst = skeleton;
        let mut after_first = skeleton;
        let chunks = self.order.len() as u64;
        let (sender, receiver) = (SENDER as usize - 1, RECEIVER as usize - 1);
        for count in 1..=recoveries {
            let session_id = SessionId {
                node: SENDER,
                incarnation: INCARNATION,
                count,
            };
            nodes[sender].process(Command::StartRecovery {
                chunks,
                sender: SENDER,
                receiver: RECEIVER,
            });
            nodes[receiver].process(Command::AdoptRecovery {
                session_id,
                chunks,
                sender: SENDER,
                receiver: RECEIVER,
            });
            held_at_worst = held_at_worst.max(self.counter.live());
            for id in &self.order {
                nodes[sender].process(Command::SendRecoveryPartial {
                    session_id,
                    first: *id,
                    last: *id,
                    sender: SENDER,
                    receiver: RECEIVER,
                });
                nodes[receiver].process(Command::ReceiveRecoveryPartial {
                    session_id,
                    first: *id,
                    last: *id,
                    sender: SENDER,
                    receiver: RECEIVER,
                });
                held_at_worst = held_at_worst.max(self.counter.live());
            }
            if count == 1 {
                after_first = self.counter.live();
            }
        }
        let profile = MemoryProfile {
            skeleton,
            held_at_worst,
            peak: self.counter.peak(),
            after_first,
            after_all: self.counter.live(),
        };
        self.counter.stop();
        profile
    }
}
