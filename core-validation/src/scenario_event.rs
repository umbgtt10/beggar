// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::session_id::SessionId;

// What passes between two nodes, as their control layers would send it: a
// request for chunks, which opens no session; a run of chunks, carrying its
// session's name and count, from which the receiver adopts the session; and a
// receiver telling the sender it gave a session up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioEvent {
    Request {
        requester: u64,
        server: u64,
        wanted: u64,
    },
    Run {
        sender: u64,
        receiver: u64,
        session_id: SessionId,
        chunks: u64,
        first: u64,
        last: u64,
    },
    Abandon {
        receiver: u64,
        sender: u64,
        session_id: SessionId,
    },
}
