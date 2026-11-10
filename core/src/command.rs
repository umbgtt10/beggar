// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::session_id::SessionId;

// What a caller asks of an instance: imperative, as every command is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    StartRecovery {
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
    ShowRegistry,
}
