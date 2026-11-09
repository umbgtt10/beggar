// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::received_chunks::ReceivedChunks;
use crate::session_id::SessionId;

// A session's record: its name, how many chunks it holds, its two ends, and
// the chunks received.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub session_id: SessionId,
    pub chunks: u64,
    pub sender: u64,
    pub receiver: u64,
    pub received: ReceivedChunks,
}
