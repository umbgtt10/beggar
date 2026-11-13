// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The chunk ids from `first` to `last`, both included (§1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    pub first: u64,
    pub last: u64,
}
