// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde_json::Value;

use crate::run::Run;
use crate::session_name::SessionName;
use crate::vocabulary::command_name::CommandName;

// A command as the specification encodes it, each of its members read where it
// carries one: the cancel's reason kept as given, since beggar never reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub name: CommandName,
    pub session_id: Option<SessionName>,
    pub chunks: Option<u64>,
    pub sender: Option<u64>,
    pub receiver: Option<u64>,
    pub run: Option<Run>,
    pub reason: Option<Value>,
}
