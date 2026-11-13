// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use alloc::vec::Vec;
use serde_json::json;
use serde_json::Value;

use crate::instance_state::InstanceState;
use crate::retained::Retained;
use crate::run::Run;
use crate::session::Session;
use crate::session_name::SessionName;

// What an instance shows of its state, in the specification's encoding: every
// u64 a string of decimal digits (§1), and the records in canonical order --
// by name, and retained ones under one name by the receipt that finished them.
pub struct View;

impl View {
    // The checkpoint (§10).
    #[must_use]
    pub fn checkpoint(state: &InstanceState) -> Value {
        let mut open: Vec<&Session> = state.open.iter().collect();
        open.sort_by_key(|session| session.name);
        let mut retained: Vec<&Retained> = state.retained.iter().collect();
        retained.sort_by_key(|kept| (kept.session.name, kept.finished));
        json!({
            "instance": {
                "node": Self::number(state.node),
                "incarnation": Self::number(state.incarnation),
            },
            "retention": Self::number(state.retention),
            "bound": state.bound.map(Self::number),
            "open": open.into_iter().map(Self::record).collect::<Vec<Value>>(),
            "retained": retained
                .into_iter()
                .map(|kept| json!({
                    "record": Self::record(&kept.session),
                    "finished": Self::number(kept.finished),
                }))
                .collect::<Vec<Value>>(),
            "name_count": Self::number(state.name_count),
            "next_receipt": Self::number(state.next_receipt),
        })
    }

    // The names of the open sessions (§9).
    #[must_use]
    pub fn names(state: &InstanceState) -> Value {
        let mut names: Vec<SessionName> = state.open.iter().map(|session| session.name).collect();
        names.sort();
        Value::Array(names.into_iter().map(Self::session_name).collect())
    }

    #[must_use]
    pub fn number(value: u64) -> Value {
        Value::String(value.to_string())
    }

    #[must_use]
    pub fn record(session: &Session) -> Value {
        json!({
            "session_id": Self::session_name(session.name),
            "chunks": Self::number(session.chunks),
            "sender": Self::number(session.sender),
            "receiver": Self::number(session.receiver),
            "chunk_set": session
                .chunk_set
                .ids()
                .into_iter()
                .map(Self::number)
                .collect::<Vec<Value>>(),
        })
    }

    #[must_use]
    pub fn run(run: Run) -> Value {
        json!({"first": Self::number(run.first), "last": Self::number(run.last)})
    }

    #[must_use]
    pub fn runs(runs: &[Run]) -> Value {
        Value::Array(runs.iter().copied().map(Self::run).collect())
    }

    #[must_use]
    pub fn session_name(name: SessionName) -> Value {
        json!({
            "node": Self::number(name.node),
            "incarnation": Self::number(name.incarnation),
            "count": Self::number(name.count),
        })
    }
}
