// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scenario_reason::ScenarioReason;
use crate::transfer::Transfer;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use beggar::checkpoint::Checkpoint;
use beggar::output::Output;
use beggar::session_id::SessionId;
use core::ops::RangeInclusive;

// What a recovery must keep across a cluster, checked against what the
// cluster knows of each transfer from outside its nodes and against what each
// node's checkpoint shows. Every check names each violation it finds. The
// checks that hold at every step make up the violations; abandonment holds
// once nothing is in flight, and liveness only where nothing was lost and
// nothing restarted.
pub struct ClusterInvariants {
    transfers: Vec<Transfer>,
    checkpoints: Vec<Checkpoint>,
}

impl ClusterInvariants {
    pub fn new(transfers: Vec<Transfer>, checkpoints: Vec<Checkpoint>) -> Self {
        Self {
            transfers,
            checkpoints,
        }
    }

    // A receiver holds a session open exactly when some of its chunks reached
    // it and not all; a sender exactly when it has not sent them all. A
    // transfer a restart interrupted, the cancellation policy ended, or a late
    // run reopened, is excepted.
    pub fn abandonment(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for transfer in self.transfers.iter().filter(|transfer| {
            !transfer.interrupted && transfer.reopens == 0 && transfer.cancellations == 0
        }) {
            let delivered = transfer.delivered.len() as u64;
            let incomplete = delivered > 0 && delivered < transfer.chunks;
            let receiving = self.holds(transfer.receiver, transfer.session_id);
            if receiving != incomplete {
                violations.push(format!(
                    "{:?}: its receiver holds it open {receiving}, with {delivered} of {} chunks delivered",
                    transfer.session_id, transfer.chunks
                ));
            }
            let sending = self.holds(transfer.sender, transfer.session_id);
            if sending == transfer.sender_completed {
                violations.push(format!(
                    "{:?}: its sender holds it open {sending}, having completed it {}",
                    transfer.session_id, transfer.sender_completed
                ));
            }
        }
        violations
    }

    // Every receipt at either end was what the chunks sent and delivered call
    // for.
    pub fn accounting(&self) -> Vec<String> {
        self.transfers
            .iter()
            .filter(|transfer| transfer.unsound > 0)
            .map(|transfer| {
                format!(
                    "{:?}: {} receipts not what its chunks called for",
                    transfer.session_id, transfer.unsound
                )
            })
            .collect()
    }

    // Every open record is of a transfer, on its terms.
    pub fn agreement(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for checkpoint in &self.checkpoints {
            for record in &checkpoint.open {
                match self.transfer(record.session_id) {
                    None => violations.push(format!(
                        "node {} holds {:?}, of no transfer",
                        checkpoint.instance.node, record.session_id
                    )),
                    Some(transfer)
                        if (record.chunks, record.sender, record.receiver)
                            != (transfer.chunks, transfer.sender, transfer.receiver) =>
                    {
                        violations.push(format!(
                            "node {} holds {:?} on other terms than its transfer's",
                            checkpoint.instance.node, record.session_id
                        ))
                    }
                    Some(_) => {}
                }
            }
        }
        violations
    }

    // A session is held by its two ends alone.
    pub fn bystanders(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for checkpoint in &self.checkpoints {
            let node = checkpoint.instance.node;
            for record in &checkpoint.open {
                if let Some(transfer) = self.transfer(record.session_id) {
                    if node != transfer.sender && node != transfer.receiver {
                        violations.push(format!(
                            "node {node} holds {:?} and is neither of its ends",
                            record.session_id
                        ));
                    }
                }
            }
        }
        violations
    }

    // Every checkpoint lists its records in ascending order of their names, no
    // two sharing one.
    pub fn canonical(&self) -> Vec<String> {
        self.checkpoints
            .iter()
            .filter(|checkpoint| {
                checkpoint
                    .open
                    .windows(2)
                    .any(|pair| pair[0].session_id >= pair[1].session_id)
            })
            .map(|checkpoint| {
                format!(
                    "node {} lists its records out of the order of their names",
                    checkpoint.instance.node
                )
            })
            .collect()
    }

    // Each transfer's receiver, asked what it lacks of the session, answers: of
    // one it holds open, the chunks that have not reached it since it opened
    // there; of one it does not, at retention 0, that it never started it, and
    // above, that or what a finished one it keeps lacked.
    pub fn gaps(&self, missing: &[Output<ScenarioReason>]) -> Vec<String> {
        let mut violations = Vec::new();
        for (transfer, answer) in self.transfers.iter().zip(missing) {
            let sound = if transfer.receiver_open {
                *answer
                    == Output::MissingChunksListed {
                        missing: Self::lacking(transfer),
                    }
            } else if self.retention(transfer.receiver) == 0 {
                *answer == Output::RecoveryNotStarted
            } else {
                matches!(
                    answer,
                    Output::RecoveryNotStarted | Output::MissingChunksListed { .. }
                )
            };
            if !sound {
                violations.push(format!(
                    "{:?}: its receiver, holding it open {}, answers {answer:?} for what it lacks, having held {:?}",
                    transfer.session_id, transfer.receiver_open, transfer.held
                ));
            }
        }
        violations
    }

    // Every transfer completed at both ends, and no session left open but one
    // a late copy reopened.
    pub fn liveness(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for transfer in &self.transfers {
            if !transfer.sender_completed {
                violations.push(format!(
                    "{:?}: its sender never completed it",
                    transfer.session_id
                ));
            }
            if transfer.receiver_completions == 0 {
                violations.push(format!(
                    "{:?}: its receiver never completed it",
                    transfer.session_id
                ));
            }
        }
        for checkpoint in &self.checkpoints {
            for record in &checkpoint.open {
                let reopened = self
                    .transfer(record.session_id)
                    .is_some_and(|transfer| transfer.reopens > 0);
                if !reopened {
                    violations.push(format!(
                        "node {} left {:?} open",
                        checkpoint.instance.node, record.session_id
                    ));
                }
            }
        }
        violations
    }

    // Every instance has minted exactly the names of the transfers it started:
    // an adopt mints none.
    pub fn names_minted(&self) -> Vec<String> {
        self.checkpoints
            .iter()
            .filter_map(|checkpoint| {
                let instance = checkpoint.instance;
                let started = self
                    .transfers
                    .iter()
                    .filter(|transfer| {
                        transfer.session_id.node == instance.node
                            && transfer.session_id.incarnation == instance.incarnation
                    })
                    .count() as u64;
                (checkpoint.name_count != started).then(|| {
                    format!(
                        "{instance:?} has minted {} names and started {started} transfers",
                        checkpoint.name_count
                    )
                })
            })
            .collect()
    }

    // A finished session reopens only by a run that reached its receiver after
    // it finished there, completed or cancelled.
    pub fn reopens(&self) -> Vec<String> {
        self.transfers
            .iter()
            .filter(|transfer| transfer.reopens > transfer.late_copies)
            .map(|transfer| {
                format!(
                    "{:?}: reopened {} times by {} late copies",
                    transfer.session_id, transfer.reopens, transfer.late_copies
                )
            })
            .collect()
    }

    // Nothing is left open anywhere: what the cancellation policy holds once
    // nothing is in flight.
    pub fn settled(&self) -> Vec<String> {
        self.checkpoints
            .iter()
            .flat_map(|checkpoint| {
                checkpoint.open.iter().map(move |record| {
                    format!(
                        "node {} still holds {:?} open",
                        checkpoint.instance.node, record.session_id
                    )
                })
            })
            .collect()
    }

    // A receiver holds only chunks delivered to it, a sender only chunks it
    // sent.
    pub fn subsets(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for checkpoint in &self.checkpoints {
            let node = checkpoint.instance.node;
            for record in &checkpoint.open {
                let Some(transfer) = self.transfer(record.session_id) else {
                    continue;
                };
                let witnessed = if node == transfer.receiver {
                    &transfer.delivered
                } else {
                    &transfer.sent
                };
                let held: BTreeSet<u64> = record.chunk_set.ids().into_iter().collect();
                if !held.is_subset(witnessed) {
                    violations.push(format!(
                        "node {node} holds chunks {held:?} of {:?}, of which only {witnessed:?} reached it",
                        record.session_id
                    ));
                }
            }
        }
        violations
    }

    // No two transfers share a name.
    pub fn unique_names(&self) -> Vec<String> {
        let mut seen = BTreeSet::new();
        self.transfers
            .iter()
            .filter(|transfer| !seen.insert(transfer.session_id))
            .map(|transfer| format!("{:?} names two transfers", transfer.session_id))
            .collect()
    }

    // Every check that holds at every step, together.
    pub fn violations(&self) -> Vec<String> {
        let mut violations = self.accounting();
        violations.extend(self.agreement());
        violations.extend(self.bystanders());
        violations.extend(self.canonical());
        violations.extend(self.names_minted());
        violations.extend(self.reopens());
        violations.extend(self.subsets());
        violations.extend(self.unique_names());
        violations
    }

    fn holds(&self, node: u64, session_id: SessionId) -> bool {
        self.checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.instance.node == node)
            .flat_map(|checkpoint| &checkpoint.open)
            .any(|record| record.session_id == session_id)
    }

    // The chunks of a transfer that have not reached its receiver since it
    // opened there, gathered into runs.
    fn lacking(transfer: &Transfer) -> Vec<RangeInclusive<u64>> {
        let mut runs: Vec<RangeInclusive<u64>> = Vec::new();
        for id in (1..=transfer.chunks).filter(|id| !transfer.held.contains(id)) {
            match runs.last_mut() {
                Some(run) if *run.end() + 1 == id => *run = *run.start()..=id,
                _ => runs.push(id..=id),
            }
        }
        runs
    }

    // The retention of the node's instance as its checkpoint shows it.
    fn retention(&self, node: u64) -> u64 {
        self.checkpoints
            .iter()
            .find(|checkpoint| checkpoint.instance.node == node)
            .map_or(0, |checkpoint| checkpoint.retention)
    }

    fn transfer(&self, session_id: SessionId) -> Option<&Transfer> {
        self.transfers
            .iter()
            .find(|transfer| transfer.session_id == session_id)
    }
}
