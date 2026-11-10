// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scenario_trace_entry::ScenarioTraceEntry;
use alloc::rc::Rc;
use alloc::vec::Vec;
use beggar::command::Command;
use beggar::instance_id::InstanceId;
use beggar::observer::Observer;
use beggar::receipt::Receipt;
use core::cell::RefCell;

// The observer a node's instance reports to: every call becomes a line of the
// node's trace, under the instance that answered it.
pub struct TraceObserver {
    instance: InstanceId,
    trace: Rc<RefCell<Vec<ScenarioTraceEntry>>>,
}

impl TraceObserver {
    pub fn new(instance: InstanceId, trace: Rc<RefCell<Vec<ScenarioTraceEntry>>>) -> Self {
        Self { instance, trace }
    }
}

impl Observer for TraceObserver {
    fn observe(&mut self, command: &Command, receipt: &Receipt) {
        self.trace.borrow_mut().push(ScenarioTraceEntry {
            instance: self.instance,
            command: command.clone(),
            receipt: receipt.clone(),
        });
    }
}
