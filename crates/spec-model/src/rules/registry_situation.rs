// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance_state::InstanceState;
use crate::lookup::Lookup;

// A situation read from the sessions the registry holds: how many are open
// against the bound, and whether one of the command's name is open or retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrySituation {
    Full,
    FullAndNotOpen,
    NotOpen,
    Open,
    NeitherOpenNorRetained,
}

impl RegistrySituation {
    #[must_use]
    pub fn holds(self, command: &Command, state: &InstanceState) -> bool {
        let open = Lookup::open(state, command.session_id).is_some();
        match self {
            RegistrySituation::Full => Lookup::is_full(state),
            RegistrySituation::FullAndNotOpen => !open && Lookup::is_full(state),
            RegistrySituation::NotOpen => !open,
            RegistrySituation::Open => open,
            RegistrySituation::NeitherOpenNorRetained => {
                Lookup::read(state, command.session_id).is_none()
            }
        }
    }
}
