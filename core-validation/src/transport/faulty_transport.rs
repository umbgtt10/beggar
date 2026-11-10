// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scenario_event::ScenarioEvent;
use crate::transport::fault_counts::FaultCounts;
use crate::transport::fault_policy::FaultPolicy;
use crate::transport::message_kind::MessageKind;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::mem::take;

const ASYMMETRIC_SALT: u64 = 0x4153_594D_4D00_0000;
const DELAY_STEPS: u64 = 3;
const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
const PARTITION_SALT: u64 = 0x5041_5254_494F_4E00;
const REORDER_MAX_STEPS: u64 = 4;

// The events in flight between the nodes, misbehaving as a seeded policy says,
// as faction's faulty transport does: partition and asymmetric cuts seeded per
// link, lasting until the transport heals; a selective fault, loss, delay,
// reordering and duplication rolled per event, in that order. A delayed or
// reordered event is held for a number of sends and deliveries and released
// then, and when nothing else is in flight whatever is held is released at
// once. Every fault injected is counted.
pub struct FaultyTransport {
    policy: FaultPolicy,
    rng: u64,
    queue: VecDeque<ScenarioEvent>,
    held: Vec<(u64, ScenarioEvent)>,
    healed: bool,
    counts: FaultCounts,
}

impl FaultyTransport {
    pub fn new(policy: FaultPolicy) -> Self {
        Self {
            policy,
            rng: policy.seed | 1,
            queue: VecDeque::new(),
            held: Vec::new(),
            healed: false,
            counts: FaultCounts::default(),
        }
    }

    pub fn counts(&self) -> FaultCounts {
        self.counts
    }

    pub fn deliver(&mut self) -> Option<ScenarioEvent> {
        self.release_due();
        if self.queue.is_empty() {
            for (_, event) in take(&mut self.held) {
                self.queue.push_back(event);
            }
        }
        self.queue.pop_front()
    }

    // Every cut lifted: what crossed a cut before is lost, what crosses after
    // is not.
    pub fn heal(&mut self) {
        self.healed = true;
    }

    pub fn in_flight(&self) -> usize {
        self.queue.len() + self.held.len()
    }

    pub fn send(&mut self, event: ScenarioEvent) {
        self.release_due();
        let (from, to) = match event {
            ScenarioEvent::Request {
                requester, server, ..
            } => (requester, server),
            ScenarioEvent::Run {
                sender, receiver, ..
            } => (sender, receiver),
        };
        if self.is_cut(
            PARTITION_SALT,
            from.min(to),
            from.max(to),
            self.policy.partition,
        ) {
            self.counts.partitioned += 1;
        } else if self.is_cut(ASYMMETRIC_SALT, from, to, self.policy.asymmetric) {
            self.counts.asymmetric += 1;
        } else if MessageKind::of(&event) == self.policy.selective_target
            && self.rolls(self.policy.selective)
        {
            self.counts.selective += 1;
        } else if self.rolls(self.policy.loss) {
            self.counts.lost += 1;
        } else if self.rolls(self.policy.delay) {
            self.counts.delayed += 1;
            self.held.push((DELAY_STEPS, event));
        } else if self.rolls(self.policy.reorder) {
            self.counts.reordered += 1;
            let lag = 1 + self.next_rand() % REORDER_MAX_STEPS;
            self.held.push((lag, event));
        } else if self.rolls(self.policy.duplication) {
            self.counts.duplicated += 1;
            self.queue.push_back(event);
            self.queue.push_back(event);
        } else {
            self.queue.push_back(event);
        }
    }

    fn is_cut(&self, salt: u64, first: u64, second: u64, percent: u8) -> bool {
        if self.healed || percent == 0 {
            return false;
        }
        if percent >= 100 {
            return true;
        }
        let mut hash = self.policy.seed ^ salt;
        hash = (hash ^ first).wrapping_mul(FNV_PRIME);
        hash = (hash ^ second).wrapping_mul(FNV_PRIME);
        hash ^= hash >> 29;
        (hash % 100) < u64::from(percent)
    }

    fn next_rand(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }

    fn release_due(&mut self) {
        for (countdown, event) in take(&mut self.held) {
            if countdown <= 1 {
                self.queue.push_back(event);
            } else {
                self.held.push((countdown - 1, event));
            }
        }
    }

    fn rolls(&mut self, percent: u8) -> bool {
        if percent == 0 {
            return false;
        }
        if percent >= 100 {
            return true;
        }
        (self.next_rand() % 100) < u64::from(percent)
    }
}
