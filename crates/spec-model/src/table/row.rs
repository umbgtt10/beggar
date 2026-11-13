// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::vocabulary::command_name::CommandName;
use crate::vocabulary::output_name::OutputName;
use crate::vocabulary::rejection_name::RejectionName;

// A row of the decision table as matrix.json states it; its situation is prose
// there, and the rule book's here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub commands: Vec<CommandName>,
    pub output: OutputName,
    pub rejection: Option<RejectionName>,
}
