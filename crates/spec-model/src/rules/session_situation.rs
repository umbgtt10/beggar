// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance_state::InstanceState;
use crate::lookup::Lookup;
use crate::session::Session;
use crate::vocabulary::command_name::CommandName;

// A situation read from the open session the command names, against what the
// command states of it; none holds where no session of the name is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionSituation {
    Alike,
    Differing,
    Misattributed,
    WrongSide,
    OutOfRange,
    Advances,
    Completes,
}

impl SessionSituation {
    #[must_use]
    pub fn holds(self, command: &Command, state: &InstanceState) -> bool {
        Lookup::open(state, command.session_id)
            .is_some_and(|session| self.holds_of(command, state.node, session))
    }

    // The run holds a chunk the chunk set does not.
    fn adds(command: &Command, session: &Session) -> bool {
        command.run.is_some_and(|run| !session.chunk_set.holds(run))
    }

    // The chunk set holds every chunk of `1..=chunks` once the run is added.
    fn fills(command: &Command, session: &Session) -> bool {
        command
            .run
            .is_some_and(|run| session.chunk_set.with(run).held() == session.chunks)
    }

    fn holds_of(self, command: &Command, node: u64, session: &Session) -> bool {
        let ends =
            (command.sender, command.receiver) == (Some(session.sender), Some(session.receiver));
        let alike = ends && command.chunks == Some(session.chunks);
        match self {
            SessionSituation::Alike => alike,
            SessionSituation::Differing => !alike,
            SessionSituation::Misattributed => !ends,
            SessionSituation::WrongSide => node == Self::wrong_end(command, session),
            SessionSituation::OutOfRange => command
                .run
                .is_some_and(|run| run.first == 0 || run.last > session.chunks),
            SessionSituation::Advances => {
                Self::adds(command, session) && !Self::fills(command, session)
            }
            SessionSituation::Completes => {
                Self::adds(command, session) && Self::fills(command, session)
            }
        }
    }

    // The end a partial is not reported on (§7): a send belongs on the
    // session's sender, a receive on its receiver.
    fn wrong_end(command: &Command, session: &Session) -> u64 {
        if command.name == CommandName::SendRecoveryPartial {
            session.receiver
        } else {
            session.sender
        }
    }
}
