// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::retained::Retained;
use crate::session::Session;

// What an instance is (§2, §10): its id, retention and bound, the sessions
// open and those retained in no particular order, the names it has minted and
// the number its next receipt takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceState {
    pub node: u64,
    pub incarnation: u64,
    pub retention: u64,
    pub bound: Option<u64>,
    pub open: Vec<Session>,
    pub retained: Vec<Retained>,
    pub name_count: u64,
    pub next_receipt: u64,
}
