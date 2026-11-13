// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance_state::InstanceState;

// A situation read from the command alone, and this instance's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSituation {
    NoChunks,
    SelfAddressed,
    NotAParty,
    OwnSession,
    EmptyRun,
}

impl CommandSituation {
    #[must_use]
    pub fn holds(self, command: &Command, state: &InstanceState) -> bool {
        let node = Some(state.node);
        match self {
            CommandSituation::NoChunks => command.chunks == Some(0),
            CommandSituation::SelfAddressed => command.sender == command.receiver,
            CommandSituation::NotAParty => command.sender != node && command.receiver != node,
            CommandSituation::OwnSession => command.session_id.is_some_and(|name| {
                name.node == state.node && name.incarnation == state.incarnation
            }),
            CommandSituation::EmptyRun => command.run.is_some_and(|run| run.last < run.first),
        }
    }
}
