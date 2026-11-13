// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use serde_json::json;
use serde_json::Value;

use crate::answer::Answer;
use crate::call::Call;
use crate::instance_state::InstanceState;
use crate::rules::rule_book::RuleBook;
use crate::spec_reader::SpecReader;
use crate::table::decision_table::DecisionTable;
use crate::view::View;

// An instance as the specification states it: created from its origin, and
// answering each command, given in the specification's encoding, with the
// receipt the first row of the decision table that applies gives it (§3) --
// numbered one more than the last, whatever the row.
pub struct Model {
    table: DecisionTable,
    state: InstanceState,
}

impl Model {
    #[must_use]
    pub fn new(table: DecisionTable, state: InstanceState) -> Self {
        Self { table, state }
    }

    pub fn process(&mut self, command: &Value) -> Answer {
        let command = SpecReader::command(command);
        let number = self.state.next_receipt;
        let row = self.table.row(&command, &self.state);
        let after = InstanceState {
            next_receipt: number + 1,
            ..RuleBook::rule(&row.id)
                .change
                .applied(&command, &self.state)
        };
        let output = row.output.encoded(
            row.rejection,
            &Call {
                command: &command,
                before: &self.state,
                after: &after,
            },
        );
        let answer = Answer {
            row: String::from(&row.id),
            receipt: json!({"number": View::number(number), "output": output}),
        };
        self.state = after;
        answer
    }
}
