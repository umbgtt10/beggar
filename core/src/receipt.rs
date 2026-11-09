// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::output::Output;

// Every call's answer: its output, and its number, one more than the
// instance's last receipt's, the first being 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub number: u64,
    pub output: Output,
}
