// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::command::Command;
use beggar::observer::Observer;
use beggar::receipt::Receipt;
use std::cell::RefCell;
use std::rc::Rc;

// An observer that keeps every call it is told of, in a list its test holds
// too, so what an instance reported can be read after the observer was handed
// to it.
pub struct RecordingObserver {
    calls: Rc<RefCell<Vec<(Command, Receipt)>>>,
}

impl RecordingObserver {
    pub fn new(calls: Rc<RefCell<Vec<(Command, Receipt)>>>) -> Self {
        Self { calls }
    }
}

impl Observer for RecordingObserver {
    fn observe(&mut self, command: &Command, receipt: &Receipt) {
        self.calls
            .borrow_mut()
            .push((command.clone(), receipt.clone()));
    }
}
