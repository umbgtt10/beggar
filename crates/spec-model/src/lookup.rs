// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance_state::InstanceState;
use crate::reading::Reading;
use crate::session::Session;
use crate::session_name::SessionName;

// The sessions an instance holds, looked up by name.
pub struct Lookup;

impl Lookup {
    // The open sessions number the bound (§2); there is none to number with
    // no bound.
    #[must_use]
    pub fn is_full(state: &InstanceState) -> bool {
        state
            .bound
            .is_some_and(|bound| u64::try_from(state.open.len()).is_ok_and(|open| open == bound))
    }

    #[must_use]
    pub fn open(state: &InstanceState, name: Option<SessionName>) -> Option<&Session> {
        state.open.iter().find(|session| Some(session.name) == name)
    }

    // The session a query reads (§9): the open one, else the retained one of
    // the name that finished last.
    #[must_use]
    pub fn read(state: &InstanceState, name: Option<SessionName>) -> Option<Reading<'_>> {
        Self::open(state, name)
            .map(|session| Reading {
                session,
                finished: None,
            })
            .or_else(|| {
                state
                    .retained
                    .iter()
                    .filter(|kept| Some(kept.session.name) == name)
                    .max_by_key(|kept| kept.finished)
                    .map(|kept| Reading {
                        session: &kept.session,
                        finished: Some(kept.finished),
                    })
            })
    }

    // The open session a command names, where an earlier row of the table has
    // already found it open.
    #[must_use]
    pub fn session(state: &InstanceState, name: Option<SessionName>) -> &Session {
        Self::open(state, name).expect("the open session the command names")
    }
}
