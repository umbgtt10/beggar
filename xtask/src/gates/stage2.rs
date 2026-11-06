// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::stage_failure::StageFailure;

pub struct Stage2<'a> {
    gates: Vec<&'a dyn Gate>,
}

impl<'a> Stage2<'a> {
    pub fn new(gates: Vec<&'a dyn Gate>) -> Self {
        Self { gates }
    }

    // Stops at the first failure rather than collecting every one: stern4rust
    // runs first because its corrections are renames and file moves, so a
    // layout it is about to reject is a layout the later gates would have
    // measured for nothing.
    pub fn run(&self) -> Result<(), StageFailure> {
        for gate in &self.gates {
            // Asked once and reused: a second call could answer differently if a
            // gate ever derives its label, which would make the failure name a
            // gate other than the one that ran.
            let label = gate.label();
            println!("{label}...");

            if let Err(failure) = gate.run() {
                return Err(StageFailure::new(&label, failure));
            }
        }

        Ok(())
    }
}
