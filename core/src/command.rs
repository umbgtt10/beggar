// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::session_id::SessionId;

// What a caller asks of an instance: imperative, as every command is. R is the
// caller's cancel-reason enumeration, which beggar records and never reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command<R> {
    StartRecovery {
        chunks: u64,
        sender: u64,
        receiver: u64,
    },
    AdoptRecovery {
        session_id: SessionId,
        chunks: u64,
        sender: u64,
        receiver: u64,
    },
    SendRecoveryPartial {
        session_id: SessionId,
        first: u64,
        last: u64,
        sender: u64,
        receiver: u64,
    },
    ReceiveRecoveryPartial {
        session_id: SessionId,
        first: u64,
        last: u64,
        sender: u64,
        receiver: u64,
    },
    CancelRecovery {
        session_id: SessionId,
        reason: R,
    },
    ShowRegistry,
}
