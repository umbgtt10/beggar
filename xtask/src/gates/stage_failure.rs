// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::gate_failure::GateFailure;
use std::fmt;

// The gate a stage stopped at, by its label, and how it failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageFailure {
    gate: String,
    failure: GateFailure,
}

impl StageFailure {
    pub fn new(gate: &str, failure: GateFailure) -> Self {
        Self {
            gate: String::from(gate),
            failure,
        }
    }
}

impl fmt::Display for StageFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.gate, self.failure)
    }
}
