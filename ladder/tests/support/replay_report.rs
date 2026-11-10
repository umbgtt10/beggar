// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::ops::RangeInclusive;

// What a replay made of a log: how many lines it held, how many it rebuilt
// receipt for receipt, the receipt numbers it found missing, and every line it
// rebuilt otherwise than logged.
pub struct ReplayReport {
    pub lines: u64,
    pub reconstructed: u64,
    pub gaps: Vec<RangeInclusive<u64>>,
    pub departures: Vec<String>,
}
