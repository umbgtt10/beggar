// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use xtask::gates::gate::Gate;
use xtask::gates::gate_failure::GateFailure;

// A gate that records whether it ran, so Stage2's ordering and its
// stop-at-the-first-failure behaviour can be observed without a real tool.
pub struct FakeGate {
    label: String,
    outcome: Result<(), GateFailure>,
    ran: RefCell<bool>,
}

impl FakeGate {
    pub fn passing(label: &str) -> Self {
        Self {
            label: String::from(label),
            outcome: Ok(()),
            ran: RefCell::new(false),
        }
    }

    pub fn failing(label: &str, failure: GateFailure) -> Self {
        Self {
            label: String::from(label),
            outcome: Err(failure),
            ran: RefCell::new(false),
        }
    }

    pub fn has_run(&self) -> bool {
        *self.ran.borrow()
    }
}

impl Gate for FakeGate {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn run(&self) -> Result<(), GateFailure> {
        *self.ran.borrow_mut() = true;
        self.outcome.clone()
    }
}
