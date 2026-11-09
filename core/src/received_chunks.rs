// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

// The chunks a session has received, shown through behaviour and never its
// fields. In this version no chunk is ever recorded, so the set is empty.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReceivedChunks {
    ids: Vec<u64>,
}

impl ReceivedChunks {
    pub fn new() -> Self {
        Self { ids: Vec::new() }
    }

    // The chunk ids received, in ascending order.
    pub fn ids(&self) -> Vec<u64> {
        self.ids.clone()
    }
}
