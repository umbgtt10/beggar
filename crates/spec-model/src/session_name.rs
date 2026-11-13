// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// A session's name (§4): the instance id that named it and a count, ordered
// as the canonical order compares them -- node, incarnation, count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionName {
    pub node: u64,
    pub incarnation: u64,
    pub count: u64,
}
