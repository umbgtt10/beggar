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

// Recoveries one after another on one instance at the retention given: each a session
// of as many chunks as the order names, sent to one peer a chunk at a time in
// that order until it completes and is evicted -- or, given a count, cancelled
// once that many chunks are sent. The counter runs from before the instance
// is created, and nothing here allocates while it does.
pub struct RecoverySequence<'a> {
    counter: &'a dyn AllocationCounter,
    order: Vec<u64>,
    cancel_after: Option<usize>,
    retention: u64,
}

impl<'a> RecoverySequence<'a> {
    pub fn new(
        counter: &'a dyn AllocationCounter,
        order: Vec<u64>,
        cancel_after: Option<usize>,
        retention: u64,
    ) -> Self {
        Self {
            counter,
            order,
            cancel_after,
            retention,
        }
    }

    pub fn run(&self, recoveries: u64) -> MemoryProfile {
        self.counter.start();
        let mut beggar: Beggar<()> = Beggar::new(
            Origin::Fresh(Fresh {
                instance: InstanceId {
                    node: NODE,
                    incarnation: INCARNATION,
                },
                retention: self.retention,
                bound: None,
            }),
            Box::new(NoOpObserver),
        );
        let skeleton = self.counter.live();
        let mut held_at_worst = skeleton;
        let mut after_full = skeleton;
        let mut peak_while_filling = skeleton;
        for count in 1..=recoveries {
            beggar.process(Command::StartRecovery {
                chunks: self.order.len() as u64,
                sender: NODE,
                receiver: PEER,
            });
            held_at_worst = held_at_worst.max(self.counter.live());
            for (sent, id) in self.order.iter().enumerate() {
                if self.cancel_after == Some(sent) {
                    beggar.process(Command::CancelRecovery {
                        session_id: Self::session_id(count),
                        reason: (),
                    });
                    break;
                }
                beggar.process(Self::send(count, *id));
                held_at_worst = held_at_worst.max(self.counter.live());
            }
            if count == self.retention.max(1) {
                after_full = self.counter.live();
                peak_while_filling = self.counter.peak();
                self.counter.rebase_peak();
            }
        }
        let profile = MemoryProfile {
            skeleton,
            held_at_worst,
            peak: peak_while_filling.max(self.counter.peak()),
            peak_once_full: self.counter.peak(),
            after_full,
            after_all: self.counter.live(),
        };
        self.counter.stop();
        profile
    }

    fn send(count: u64, id: u64) -> Command<()> {
        Command::SendRecoveryPartial {
            session_id: Self::session_id(count),
            first: id,
            last: id,
            sender: NODE,
            receiver: PEER,
        }
    }

    fn session_id(count: u64) -> SessionId {
        SessionId {
            node: NODE,
            incarnation: INCARNATION,
            count,
        }
    }
}
