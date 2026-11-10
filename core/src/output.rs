// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::checkpoint::Checkpoint;
use crate::rejection::Rejection;
use crate::session_id::SessionId;
use crate::session_record::SessionRecord;
use alloc::vec::Vec;
use core::ops::RangeInclusive;

// What happened: past participle, as every output is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output<R> {
    RecoveryStarted {
        session_id: SessionId,
    },
    RecoveryAdvanced {
        held: u64,
        chunks: u64,
    },
    RecoveryCompleted,
    RecoveryCancelled(R),
    RecoveryUnaffected,
    RecoveryNotStarted,
    RecoveryConflicted {
        chunks: u64,
        sender: u64,
        receiver: u64,
    },
    RecoveryRejected(Rejection),
    RecoveriesListed {
        sessions: Vec<SessionId>,
    },
    RecoveryShown {
        record: SessionRecord,
        finished: Option<u64>,
    },
    MissingChunksListed {
        missing: Vec<RangeInclusive<u64>>,
    },
    RegistryShown(Checkpoint),
}
