// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::rejection::Rejection;

// The rows a start meets before a session is opened, in the matrix's order:
// its chunks, its ends, this node's part in it, and the bound.
pub struct StartRows {
    node: u64,
    open: u64,
    bound: Option<u64>,
}

impl StartRows {
    pub fn new(node: u64, open: u64, bound: Option<u64>) -> Self {
        Self { node, open, bound }
    }

    // The first row that refuses the start, or none.
    pub fn refusal(&self, chunks: u64, sender: u64, receiver: u64) -> Option<Rejection> {
        if chunks == 0 {
            Some(Rejection::NoChunks)
        } else if sender == receiver {
            Some(Rejection::SelfAddressed)
        } else if self.node != sender && self.node != receiver {
            Some(Rejection::NotAParty)
        } else {
            self.bound
                .filter(|bound| self.open >= *bound)
                .map(|capacity| Rejection::RegistryFull { capacity })
        }
    }
}
