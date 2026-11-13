// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// A session's name: the instance id that named it, and a count. Ordered as
// the specification orders records: node, incarnation, count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionId {
    pub node: u64,
    pub incarnation: u64,
    pub count: u64,
}
