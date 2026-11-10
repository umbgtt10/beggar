// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::checkpoint::Checkpoint;
use crate::chunk_set::ChunkSet;
use crate::opening_rows::OpeningRows;
use crate::output::Output;
use crate::partial_rows::PartialRows;
use crate::session_id::SessionId;
use crate::session_record::SessionRecord;

// The registry: its state, kept in the checkpoint's own shape so ShowRegistry
// answers it as it stands, and what a command does to it -- a start or an
// adopt opens a session, a partial updates one, a completion evicts it -- with
// the rows each command meets before it does anything.
pub struct Registry {
    state: Checkpoint,
}

impl Registry {
    pub fn new(state: Checkpoint) -> Self {
        Self { state }
    }

    // A run added to the open session at the index, and what that did: none
    // new leaves the session unaffected, a full set completes and evicts it,
    // and anything else advances it.
    pub fn account(&mut self, index: usize, first: u64, last: u64) -> Output {
        let record = &mut self.state.open[index];
        let before = record.chunk_set.count();
        record.chunk_set.add(first, last);
        let held = record.chunk_set.count();
        let chunks = record.chunks;
        if held == before {
            Output::RecoveryUnaffected
        } else if held == chunks {
            self.state.open.remove(index);
            Output::RecoveryCompleted
        } else {
            Output::RecoveryAdvanced { held, chunks }
        }
    }

    // The registry as it stands: what ShowRegistry answers.
    pub fn checkpoint(&self) -> Checkpoint {
        self.state.clone()
    }

    // A call counted: its receipt's number, one more than the last.
    pub fn count_call(&mut self) -> u64 {
        let number = self.state.next_receipt;
        self.state.next_receipt = number + 1;
        number
    }

    // The next name this instance mints.
    pub fn mint(&mut self) -> SessionId {
        self.state.name_count += 1;
        SessionId {
            node: self.state.instance.node,
            incarnation: self.state.instance.incarnation,
            count: self.state.name_count,
        }
    }

    // A session opened, with an empty chunk set, in the order of the names.
    pub fn open(&mut self, session_id: SessionId, chunks: u64, sender: u64, receiver: u64) {
        let at = self
            .state
            .open
            .partition_point(|record| record.session_id < session_id);
        self.state.open.insert(
            at,
            SessionRecord {
                session_id,
                chunks,
                sender,
                receiver,
                chunk_set: ChunkSet::new(),
            },
        );
    }

    pub fn opening_rows(&self) -> OpeningRows<'_> {
        OpeningRows::new(self.state.instance, &self.state.open, self.state.bound)
    }

    pub fn partial_rows(&self) -> PartialRows<'_> {
        PartialRows::new(self.state.instance.node, &self.state.open)
    }
}
