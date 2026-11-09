// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::received_chunks::ReceivedChunks;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;

// A second implementation of the specification, deliberately naive and
// shaped otherwise than the crate: the matrix read as a table, its rows tried
// top to bottom, the first that applies answering. What the two agree on is
// what the specification says, not what one of them happens to do.
pub struct NaiveRegistry {
    instance: InstanceId,
    retention: u64,
    bound: Option<u64>,
    open: Vec<SessionRecord>,
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
            open: checkpoint.open.clone(),
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
            open: self.open.clone(),
            retained: Vec::new(),
            name_count: self.names_minted,
            next_receipt: self.receipts_issued + 1,
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
        self.open.push(SessionRecord {
            session_id,
            chunks,
            sender,
            receiver,
            received: ReceivedChunks::new(),
        });
        Output::RecoveryStarted { session_id }
    }
}
