// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance_id::InstanceId;
use crate::output::Output;
use crate::rejection::Rejection;
use crate::session_id::SessionId;
use crate::session_record::SessionRecord;

// The rows a session meets before it is opened, in the matrix's order: a
// start's and an adopt's form -- its chunks, its ends, this node's part in it
// -- then, for an adopt, its name and the session open under it, and for both
// the bound.
pub struct OpeningRows<'a> {
    instance: InstanceId,
    open: &'a [SessionRecord],
    bound: Option<u64>,
}

impl<'a> OpeningRows<'a> {
    pub fn new(instance: InstanceId, open: &'a [SessionRecord], bound: Option<u64>) -> Self {
        Self {
            instance,
            open,
            bound,
        }
    }

    // The answer of the first row that meets an adopt before a session is
    // opened for it, or none: then it is opened.
    pub fn adoption<R>(
        &self,
        session_id: SessionId,
        chunks: u64,
        sender: u64,
        receiver: u64,
    ) -> Option<Output<R>> {
        if let Some(rejection) = self.form(chunks, sender, receiver) {
            return Some(Output::RecoveryRejected(rejection));
        }
        if session_id.node == self.instance.node
            && session_id.incarnation == self.instance.incarnation
        {
            return Some(Output::RecoveryRejected(Rejection::OwnSession));
        }
        if let Some(record) = self
            .open
            .iter()
            .find(|record| record.session_id == session_id)
        {
            let same =
                record.chunks == chunks && record.sender == sender && record.receiver == receiver;
            return Some(if same {
                Output::RecoveryUnaffected
            } else {
                Output::RecoveryConflicted {
                    chunks: record.chunks,
                    sender: record.sender,
                    receiver: record.receiver,
                }
            });
        }
        self.full().map(Output::RecoveryRejected)
    }

    // The first row that refuses a start, or none.
    pub fn start_refusal(&self, chunks: u64, sender: u64, receiver: u64) -> Option<Rejection> {
        self.form(chunks, sender, receiver).or_else(|| self.full())
    }

    // The rows a start and an adopt share: its chunks, its ends, and this
    // node's part in it.
    fn form(&self, chunks: u64, sender: u64, receiver: u64) -> Option<Rejection> {
        if chunks == 0 {
            Some(Rejection::NoChunks)
        } else if sender == receiver {
            Some(Rejection::SelfAddressed)
        } else if self.instance.node != sender && self.instance.node != receiver {
            Some(Rejection::NotAParty)
        } else {
            None
        }
    }

    fn full(&self) -> Option<Rejection> {
        self.bound
            .filter(|bound| self.open.len() as u64 >= *bound)
            .map(|capacity| Rejection::RegistryFull { capacity })
    }
}
