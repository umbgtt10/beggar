// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// How many times the transport injected each fault: the evidence that a run
// misbehaved as its policy told it to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FaultCounts {
    pub lost: u64,
    pub duplicated: u64,
    pub delayed: u64,
    pub reordered: u64,
    pub partitioned: u64,
    pub asymmetric: u64,
    pub selective: u64,
}
