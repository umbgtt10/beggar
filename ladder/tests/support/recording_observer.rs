// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::command::Command;
use beggar::observer::Observer;
use beggar::receipt::Receipt;
use beggar_core_validation::scenario_reason::ScenarioReason;
use std::cell::RefCell;
use std::rc::Rc;

// The calls an observer was told of, each with its receipt, shared with the
// test that reads them.
pub type Calls = Rc<RefCell<Vec<(Command<ScenarioReason>, Receipt<ScenarioReason>)>>>;

// An observer that keeps every call it is told of, in a list its test holds
// too, so what an instance reported can be read after the observer was handed
// to it.
pub struct RecordingObserver {
    calls: Calls,
}

impl RecordingObserver {
    pub fn new(calls: Calls) -> Self {
        Self { calls }
    }
}

impl Observer<ScenarioReason> for RecordingObserver {
    fn observe(&mut self, command: &Command<ScenarioReason>, receipt: &Receipt<ScenarioReason>) {
        self.calls
            .borrow_mut()
            .push((command.clone(), receipt.clone()));
    }
}
