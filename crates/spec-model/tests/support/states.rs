// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_spec_model::chunk_set::ChunkSet;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::retained::Retained;
use beggar_spec_model::session::Session;
use beggar_spec_model::session_name::SessionName;

// Node 2, incarnation 3, keeping one finished session and two open at most:
// two names minted, receipt 5 next.
pub struct States;

impl States {
    pub fn instance() -> InstanceState {
        InstanceState {
            node: 2,
            incarnation: 3,
            retention: 1,
            bound: Some(2),
            open: Vec::new(),
            retained: Vec::new(),
            name_count: 2,
            next_receipt: 5,
        }
    }

    pub fn holding(open: Vec<Session>, retained: Vec<Retained>) -> InstanceState {
        InstanceState {
            open,
            retained,
            ..Self::instance()
        }
    }

    pub fn own(count: u64) -> SessionName {
        SessionName {
            node: 2,
            incarnation: 3,
            count,
        }
    }

    pub fn peers(count: u64) -> SessionName {
        SessionName {
            node: 1,
            incarnation: 7,
            count,
        }
    }

    pub fn retained(name: SessionName, finished: u64) -> Retained {
        Retained {
            session: Self::session(name, &[1, 3]),
            finished,
        }
    }

    // Four chunks, from node 1 to node 2, the ids given held.
    pub fn session(name: SessionName, ids: &[u64]) -> Session {
        Session {
            name,
            chunks: 4,
            sender: 1,
            receiver: 2,
            chunk_set: ChunkSet::of(ids),
        }
    }
}
