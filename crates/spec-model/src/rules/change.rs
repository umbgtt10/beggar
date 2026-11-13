// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::chunk_set::ChunkSet;
use crate::command::Command;
use crate::instance_state::InstanceState;
use crate::retained::Retained;
use crate::session::Session;
use crate::session_name::SessionName;

// What a row does to the instance, as its prose states it. A refusal, a query
// and every row whose output says nothing happened change nothing; the
// receipt's number is the model's to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Nothing,
    SessionStarted,
    SessionAdopted,
    RunAdded,
    SessionCompleted,
    SessionCancelled,
}

impl Change {
    #[must_use]
    pub fn applied(self, command: &Command, state: &InstanceState) -> InstanceState {
        match self {
            Change::Nothing => state.clone(),
            Change::SessionStarted => {
                let name_count = state.name_count + 1;
                let name = SessionName {
                    node: state.node,
                    incarnation: state.incarnation,
                    count: name_count,
                };
                InstanceState {
                    open: Self::opened(command, name, state),
                    name_count,
                    ..state.clone()
                }
            }
            Change::SessionAdopted => InstanceState {
                open: Self::opened(command, command.session_id.expect("a name"), state),
                ..state.clone()
            },
            Change::RunAdded => InstanceState {
                open: state
                    .open
                    .iter()
                    .map(|session| Self::added(command, session))
                    .collect(),
                ..state.clone()
            },
            Change::SessionCompleted => {
                Self::finished(command, &Change::RunAdded.applied(command, state))
            }
            Change::SessionCancelled => Self::finished(command, state),
        }
    }

    fn added(command: &Command, session: &Session) -> Session {
        if Some(session.name) == command.session_id {
            Session {
                chunk_set: session.chunk_set.with(command.run.expect("a run")),
                ..session.clone()
            }
        } else {
            session.clone()
        }
    }

    // The session named evicted (§2), and kept as it stood with this call's
    // number; when the retained then outnumber the retention, the one that
    // finished earliest is let go.
    fn finished(command: &Command, state: &InstanceState) -> InstanceState {
        let (finishing, open): (Vec<Session>, Vec<Session>) = state
            .open
            .iter()
            .cloned()
            .partition(|session| Some(session.name) == command.session_id);
        let kept: Vec<Retained> = state
            .retained
            .iter()
            .cloned()
            .chain(finishing.into_iter().map(|session| Retained {
                session,
                finished: state.next_receipt,
            }))
            .collect();
        let over = u64::try_from(kept.len()).is_ok_and(|count| count > state.retention);
        let earliest = kept.iter().map(|retained| retained.finished).min();
        InstanceState {
            open,
            retained: kept
                .into_iter()
                .filter(|retained| !over || Some(retained.finished) != earliest)
                .collect(),
            ..state.clone()
        }
    }

    // The open sessions and one more, under the name, as the command states it,
    // holding no chunk yet.
    fn opened(command: &Command, name: SessionName, state: &InstanceState) -> Vec<Session> {
        let session = Session {
            name,
            chunks: command.chunks.expect("a count"),
            sender: command.sender.expect("a sender"),
            receiver: command.receiver.expect("a receiver"),
            chunk_set: ChunkSet::default(),
        };
        state.open.iter().cloned().chain([session]).collect()
    }
}
