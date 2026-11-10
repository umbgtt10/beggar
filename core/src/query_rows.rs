// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::output::Output;
use crate::retained_record::RetainedRecord;
use crate::session_id::SessionId;
use crate::session_record::SessionRecord;

// The rows of the queries that read sessions: the open ones listed, and one
// session read by its name -- the open one if there is one, otherwise the
// retained one that finished last.
pub struct QueryRows<'a> {
    open: &'a [SessionRecord],
    retained: &'a [RetainedRecord],
}

impl<'a> QueryRows<'a> {
    pub fn new(open: &'a [SessionRecord], retained: &'a [RetainedRecord]) -> Self {
        Self { open, retained }
    }

    // The chunks the session of the name does not hold, as their maximal runs.
    pub fn missing_chunks<R>(&self, session_id: SessionId) -> Output<R> {
        self.read(session_id)
            .map_or(Output::RecoveryNotStarted, |(record, _)| {
                Output::MissingChunksListed {
                    missing: record.chunk_set.missing(record.chunks),
                }
            })
    }

    // The names of the open sessions, in the order the registry keeps them:
    // the order of the names.
    pub fn recoveries<R>(&self) -> Output<R> {
        Output::RecoveriesListed {
            sessions: self.open.iter().map(|record| record.session_id).collect(),
        }
    }

    pub fn recovery<R>(&self, session_id: SessionId) -> Output<R> {
        self.read(session_id)
            .map_or(Output::RecoveryNotStarted, |(record, finished)| {
                Output::RecoveryShown {
                    record: record.clone(),
                    finished,
                }
            })
    }

    // The session of the name and the number of the receipt that finished it:
    // the open one, unfinished, otherwise the retained one that finished last.
    fn read(&self, session_id: SessionId) -> Option<(&SessionRecord, Option<u64>)> {
        let open = self
            .open
            .iter()
            .find(|record| record.session_id == session_id);
        open.map(|record| (record, None)).or_else(|| {
            self.retained
                .iter()
                .filter(|kept| kept.record.session_id == session_id)
                .max_by_key(|kept| kept.finished)
                .map(|kept| (&kept.record, Some(kept.finished)))
        })
    }
}
