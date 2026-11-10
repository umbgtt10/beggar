// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::output::Output;
use crate::rejection::Rejection;
use crate::session_id::SessionId;
use crate::session_record::SessionRecord;
use crate::side::Side;

// The rows a partial meets before its chunks are counted, in the matrix's
// order: the run, the session, its ends, its side, its range.
pub struct PartialRows<'a> {
    node: u64,
    open: &'a [SessionRecord],
}

impl<'a> PartialRows<'a> {
    pub fn new(node: u64, open: &'a [SessionRecord]) -> Self {
        Self { node, open }
    }

    // The session's place among the open ones when no row answers the partial
    // first, or the output of the first row that does.
    pub fn attribution(
        &self,
        side: Side,
        session_id: SessionId,
        first: u64,
        last: u64,
        sender: u64,
        receiver: u64,
    ) -> Result<usize, Output> {
        if last < first {
            return Err(Output::RecoveryRejected(Rejection::EmptyRun {
                first,
                last,
            }));
        }
        let index = self
            .open
            .iter()
            .position(|record| record.session_id == session_id)
            .ok_or(Output::RecoveryNotStarted)?;
        let record = &self.open[index];
        let wrong_end = match side {
            Side::Sending => record.receiver,
            Side::Receiving => record.sender,
        };
        let rejection = if record.sender != sender || record.receiver != receiver {
            Some(Rejection::Misattributed {
                sender: record.sender,
                receiver: record.receiver,
            })
        } else if self.node == wrong_end {
            Some(Rejection::WrongSide)
        } else if first == 0 || last > record.chunks {
            Some(Rejection::ChunkOutOfRange {
                first,
                last,
                chunks: record.chunks,
            })
        } else {
            None
        };
        rejection.map_or(Ok(index), |rejection| {
            Err(Output::RecoveryRejected(rejection))
        })
    }
}
