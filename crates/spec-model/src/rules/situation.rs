// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance_state::InstanceState;
use crate::rules::command_situation::CommandSituation;
use crate::rules::registry_situation::RegistrySituation;
use crate::rules::session_situation::SessionSituation;

// When a row applies, as its prose states it: always, or by what the command
// carries, by which sessions the registry holds, or by the open session the
// command names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situation {
    Always,
    Command(CommandSituation),
    Registry(RegistrySituation),
    Session(SessionSituation),
}

impl Situation {
    #[must_use]
    pub fn holds(self, command: &Command, state: &InstanceState) -> bool {
        match self {
            Situation::Always => true,
            Situation::Command(situation) => situation.holds(command, state),
            Situation::Registry(situation) => situation.holds(command, state),
            Situation::Session(situation) => situation.holds(command, state),
        }
    }
}
