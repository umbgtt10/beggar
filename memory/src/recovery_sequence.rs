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

const INCARNATION: u64 = 1;
const NODE: u64 = 1;
const PEER: u64 = 2;

// Recoveries one after another on one instance at retention 0: each a session
// of as many chunks as the order names, sent to one peer a chunk at a time in
// that order until it completes and is evicted. The counter runs from before
// the instance is created, and nothing here allocates while it does.
pub struct RecoverySequence<'a> {
    counter: &'a dyn AllocationCounter,
    order: Vec<u64>,
}

impl<'a> RecoverySequence<'a> {
    pub fn new(counter: &'a dyn AllocationCounter, order: Vec<u64>) -> Self {
        Self { counter, order }
    }

    pub fn run(&self, recoveries: u64) -> MemoryProfile {
        self.counter.start();
        let mut beggar: Beggar<()> = Beggar::new(
            Origin::Fresh(Fresh {
                instance: InstanceId {
                    node: NODE,
                    incarnation: INCARNATION,
                },
                retention: 0,
                bound: None,
            }),
            Box::new(NoOpObserver),
        );
        let skeleton = self.counter.live();
        let mut held_at_worst = skeleton;
        let mut after_first = skeleton;
        for count in 1..=recoveries {
            beggar.process(Command::StartRecovery {
                chunks: self.order.len() as u64,
                sender: NODE,
                receiver: PEER,
            });
            held_at_worst = held_at_worst.max(self.counter.live());
            for id in &self.order {
                beggar.process(Self::send(count, *id));
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

    fn send(count: u64, id: u64) -> Command {
        Command::SendRecoveryPartial {
            session_id: SessionId {
                node: NODE,
                incarnation: INCARNATION,
                count,
            },
            first: id,
            last: id,
            sender: NODE,
            receiver: PEER,
        }
    }
}
