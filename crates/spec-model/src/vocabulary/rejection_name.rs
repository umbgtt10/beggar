// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde_json::json;
use serde_json::Value;

use crate::call::Call;
use crate::lookup::Lookup;
use crate::view::View;
use crate::vocabulary::speller::Speller;
use crate::vocabulary::spelling::Spelling;
use crate::vocabulary::variant::Variant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionName {
    NoChunks,
    SelfAddressed,
    NotAParty,
    OwnSession,
    RegistryFull,
    EmptyRun,
    Misattributed,
    WrongSide,
    ChunkOutOfRange,
}

impl RejectionName {
    pub const SPELLING: Spelling<RejectionName> = Spelling {
        kind: "a rejection",
        words: &[
            (RejectionName::NoChunks, "NoChunks"),
            (RejectionName::SelfAddressed, "SelfAddressed"),
            (RejectionName::NotAParty, "NotAParty"),
            (RejectionName::OwnSession, "OwnSession"),
            (RejectionName::RegistryFull, "RegistryFull"),
            (RejectionName::EmptyRun, "EmptyRun"),
            (RejectionName::Misattributed, "Misattributed"),
            (RejectionName::WrongSide, "WrongSide"),
            (RejectionName::ChunkOutOfRange, "ChunkOutOfRange"),
        ],
    };

    // What a refusal carries (§5-§7): the bound, the run, or the open
    // session's ends or count -- a refusal changes nothing, so the state is
    // the one the command met.
    #[must_use]
    pub fn encoded(self, call: &Call) -> Value {
        let name = Speller::name(&Self::SPELLING, self);
        let run = call.command.run;
        match self {
            RejectionName::NoChunks
            | RejectionName::SelfAddressed
            | RejectionName::NotAParty
            | RejectionName::OwnSession
            | RejectionName::WrongSide => Variant::unit(name),
            RejectionName::RegistryFull => Variant::of(
                name,
                json!({"capacity": call.before.bound.map(View::number)}),
            ),
            RejectionName::EmptyRun => Variant::of(name, View::run(run.expect("a run"))),
            RejectionName::Misattributed => {
                let session = Lookup::session(call.before, call.command.session_id);
                Variant::of(
                    name,
                    json!({
                        "sender": View::number(session.sender),
                        "receiver": View::number(session.receiver),
                    }),
                )
            }
            RejectionName::ChunkOutOfRange => {
                let mut payload = View::run(run.expect("a run"));
                payload["chunks"] =
                    View::number(Lookup::session(call.before, call.command.session_id).chunks);
                Variant::of(name, payload)
            }
        }
    }
}
