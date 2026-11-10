// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::chunk_set::ChunkSet;
use crate::session_id::SessionId;

// A session's record: its name, how many chunks it holds, its two ends, and
// its chunk set -- the chunks accounted at this end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub session_id: SessionId,
    pub chunks: u64,
    pub sender: u64,
    pub receiver: u64,
    pub chunk_set: ChunkSet,
}
