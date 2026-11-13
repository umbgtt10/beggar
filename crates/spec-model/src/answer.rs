// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use serde_json::Value;

// What the model answered a command with: the receipt, in the specification's
// encoding, and the row of the decision table that decided it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub row: String,
    pub receipt: Value,
}
