// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What one scoring tool reported of one crate: its own release, and the total
// it was asked for.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolScore {
    pub version: String,
    pub total: f64,
}
