// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::gate_failure::GateFailure;

pub trait Gate {
    fn label(&self) -> String;

    fn run(&self) -> Result<(), GateFailure>;
}
