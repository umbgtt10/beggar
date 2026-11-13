// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::chunk_set::ChunkSet;
use crate::session_name::SessionName;

// A session's record (§10): its name, its count, its ends and the chunks it has
// accounted at this end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub name: SessionName,
    pub chunks: u64,
    pub sender: u64,
    pub receiver: u64,
    pub chunk_set: ChunkSet,
}
