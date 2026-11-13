// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::command::Command;
use crate::instance_state::InstanceState;
use crate::rules::rule_book::RuleBook;
use crate::table::row::Row;

// The decision table: its rows in the order they are checked. A command is
// answered by the first row for it that applies (§3).
pub struct DecisionTable {
    rows: Vec<Row>,
}

impl DecisionTable {
    #[must_use]
    pub fn new(rows: Vec<Row>) -> Self {
        Self { rows }
    }

    #[must_use]
    pub fn row(&self, command: &Command, state: &InstanceState) -> &Row {
        self.rows
            .iter()
            .find(|row| {
                row.commands.contains(&command.name)
                    && RuleBook::rule(&row.id).situation.holds(command, state)
            })
            .expect("a row that applies: the table answers every command in every state")
    }
}
