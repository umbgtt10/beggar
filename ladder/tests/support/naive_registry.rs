// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::checkpoint::Checkpoint;
use beggar::chunk_set::ChunkSet;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use std::collections::BTreeSet;
use std::ops::RangeInclusive;

// A second implementation of the specification, deliberately naive and
// shaped otherwise than the crate: the matrix read as a table, its rows tried
// top to bottom, the first that applies answering, and a run walked chunk by
// chunk -- so it is for small chunk counts only. What the two agree on is what
// the specification says, not what one of them happens to do.
pub struct NaiveRegistry {
    instance: InstanceId,
    retention: u64,
    bound: Option<u64>,
    open: Vec<NaiveSession>,
    names_minted: u64,
    receipts_issued: u64,
}

impl NaiveRegistry {
    pub fn fresh(fresh: &Fresh) -> Self {
        Self {
            instance: fresh.instance,
            retention: fresh.retention,
            bound: fresh.bound,
            open: Vec::new(),
            names_minted: 0,
            receipts_issued: 0,
        }
    }

    pub fn restored(checkpoint: &Checkpoint) -> Self {
        Self {
            instance: checkpoint.instance,
            retention: checkpoint.retention,
            bound: checkpoint.bound,
            open: checkpoint
                .open
                .iter()
                .map(|record| NaiveSession {
                    session_id: record.session_id,
                    chunks: record.chunks,
                    sender: record.sender,
                    receiver: record.receiver,
                    ids: record.chunk_set.ids().into_iter().collect(),
                })
                .collect(),
            names_minted: checkpoint.name_count,
            receipts_issued: checkpoint.next_receipt - 1,
        }
    }

    pub fn answer(&mut self, command: &Command) -> Receipt {
        self.receipts_issued += 1;
        let output = match command {
            Command::StartRecovery {
                chunks,
                sender,
                receiver,
            } => self.start(*chunks, *sender, *receiver),
            Command::SendRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            } => self.send(*session_id, *first..=*last, (*sender, *receiver)),
            Command::ShowRegistry => Output::RegistryShown(self.checkpoint()),
        };
        Receipt {
            number: self.receipts_issued,
            output,
        }
    }

    fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            instance: self.instance,
            retention: self.retention,
            bound: self.bound,
            open: self
                .open
                .iter()
                .map(|session| {
                    let mut chunk_set = ChunkSet::new();
                    for id in &session.ids {
                        chunk_set.add(*id, *id);
                    }
                    SessionRecord {
                        session_id: session.session_id,
                        chunks: session.chunks,
                        sender: session.sender,
                        receiver: session.receiver,
                        chunk_set,
                    }
                })
                .collect(),
            retained: Vec::new(),
            name_count: self.names_minted,
            next_receipt: self.receipts_issued + 1,
        }
    }

    fn send(
        &mut self,
        session_id: SessionId,
        run: RangeInclusive<u64>,
        ends: (u64, u64),
    ) -> Output {
        let (first, last) = (*run.start(), *run.end());
        if last < first {
            return Output::RecoveryRejected(Rejection::EmptyRun { first, last });
        }
        let Some(index) = self
            .open
            .iter()
            .position(|session| session.session_id == session_id)
        else {
            return Output::RecoveryNotStarted;
        };
        let node = self.instance.node;
        let session = &mut self.open[index];
        let rows = [
            (
                ends != (session.sender, session.receiver),
                Rejection::Misattributed {
                    sender: session.sender,
                    receiver: session.receiver,
                },
            ),
            (node == session.receiver, Rejection::WrongSide),
            (
                first == 0 || last > session.chunks,
                Rejection::ChunkOutOfRange {
                    first,
                    last,
                    chunks: session.chunks,
                },
            ),
        ];
        if let Some((_, rejection)) = rows.into_iter().find(|(applies, _)| *applies) {
            return Output::RecoveryRejected(rejection);
        }
        let new: Vec<u64> = run.filter(|id| !session.ids.contains(id)).collect();
        if new.is_empty() {
            return Output::RecoveryUnaffected;
        }
        session.ids.extend(new);
        let held = session.ids.len() as u64;
        if held == session.chunks {
            self.open.remove(index);
            return Output::RecoveryCompleted;
        }
        Output::RecoveryAdvanced {
            held,
            chunks: session.chunks,
        }
    }

    fn start(&mut self, chunks: u64, sender: u64, receiver: u64) -> Output {
        let node = self.instance.node;
        let full = self
            .bound
            .is_some_and(|bound| self.open.len() as u64 == bound);
        let rows = [
            (chunks == 0, Rejection::NoChunks),
            (sender == receiver, Rejection::SelfAddressed),
            (sender != node && receiver != node, Rejection::NotAParty),
            (
                full,
                Rejection::RegistryFull {
                    capacity: self.bound.unwrap_or(0),
                },
            ),
        ];
        if let Some((_, rejection)) = rows.into_iter().find(|(applies, _)| *applies) {
            return Output::RecoveryRejected(rejection);
        }
        self.names_minted += 1;
        let session_id = SessionId {
            node,
            incarnation: self.instance.incarnation,
            count: self.names_minted,
        };
        self.open.push(NaiveSession {
            session_id,
            chunks,
            sender,
            receiver,
            ids: BTreeSet::new(),
        });
        Output::RecoveryStarted { session_id }
    }
}

// A session as the naive model keeps it: its record's numbers, and its chunk
// ids one by one in an ordered set -- nothing of the ranges the crate keeps.
struct NaiveSession {
    session_id: SessionId,
    chunks: u64,
    sender: u64,
    receiver: u64,
    ids: BTreeSet<u64>,
}
