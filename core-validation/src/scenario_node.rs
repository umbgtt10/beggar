// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

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

// One node of a scenario: the instance it runs now, fresh at retention 0 with
// no bound, and the trace of every call made to any of its instances. A
// restart replaces the instance by a new one, empty, under the next
// incarnation; the trace carries on.
pub struct ScenarioNode {
    node: u64,
    incarnation: u64,
    beggar: Beggar<()>,
    trace: Rc<RefCell<Vec<ScenarioTraceEntry>>>,
}

impl ScenarioNode {
    pub fn new(node: u64) -> Self {
        let trace = Rc::new(RefCell::new(Vec::new()));
        Self {
            node,
            incarnation: 1,
            beggar: Self::fresh(node, 1, &trace),
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

    pub fn process(&mut self, command: Command) -> Receipt {
        self.beggar.process(command)
    }

    pub fn restart(&mut self) {
        self.incarnation += 1;
        self.beggar = Self::fresh(self.node, self.incarnation, &self.trace);
    }

    pub fn trace(&self) -> Vec<ScenarioTraceEntry> {
        self.trace.borrow().clone()
    }

    fn fresh(
        node: u64,
        incarnation: u64,
        trace: &Rc<RefCell<Vec<ScenarioTraceEntry>>>,
    ) -> Beggar<()> {
        let instance = InstanceId { node, incarnation };
        Beggar::new(
            Origin::Fresh(Fresh {
                instance,
                retention: 0,
                bound: None,
            }),
            Box::new(TraceObserver::new(instance, Rc::clone(trace))),
        )
    }
}
