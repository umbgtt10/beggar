// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scenario_reason::ScenarioReason;
use crate::scenario_trace_entry::ScenarioTraceEntry;
use crate::trace_observer::TraceObserver;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use beggar::beggar::Beggar;
use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::origin::Origin;
use beggar::output::Output;
use beggar::receipt::Receipt;
use core::cell::RefCell;

// One node of a scenario: the instance it runs now, fresh at its retention
// with no bound, and the trace of every call made to any of its instances. A
// restart replaces the instance by a new one, empty, under the next
// incarnation and at the same retention; the trace carries on.
pub struct ScenarioNode {
    node: u64,
    incarnation: u64,
    retention: u64,
    beggar: Beggar<ScenarioReason>,
    trace: Rc<RefCell<Vec<ScenarioTraceEntry>>>,
}

impl ScenarioNode {
    pub fn new(node: u64, retention: u64) -> Self {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let instance = InstanceId {
            node,
            incarnation: 1,
        };
        Self {
            node,
            incarnation: 1,
            retention,
            beggar: Self::fresh(instance, retention, &trace),
            trace,
        }
    }

    // The checkpoint its instance answers now, asked as any caller asks it.
    pub fn checkpoint(&mut self) -> Checkpoint {
        match self.process(Command::ShowRegistry).output {
            Output::RegistryShown(checkpoint) => checkpoint,
            other => panic!("ShowRegistry answered {other:?}"),
        }
    }

    pub fn instance(&self) -> InstanceId {
        InstanceId {
            node: self.node,
            incarnation: self.incarnation,
        }
    }

    pub fn process(&mut self, command: Command<ScenarioReason>) -> Receipt<ScenarioReason> {
        self.beggar.process(command)
    }

    pub fn restart(&mut self) {
        self.incarnation += 1;
        self.beggar = Self::fresh(self.instance(), self.retention, &self.trace);
    }

    pub fn trace(&self) -> Vec<ScenarioTraceEntry> {
        self.trace.borrow().clone()
    }

    fn fresh(
        instance: InstanceId,
        retention: u64,
        trace: &Rc<RefCell<Vec<ScenarioTraceEntry>>>,
    ) -> Beggar<ScenarioReason> {
        Beggar::new(
            Origin::Fresh(Fresh {
                instance,
                retention,
                bound: None,
            }),
            Box::new(TraceObserver::new(instance, Rc::clone(trace))),
        )
    }
}
