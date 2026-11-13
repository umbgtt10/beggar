// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde_json::json;
use serde_json::Map;
use serde_json::Value;

use crate::call::Call;
use crate::lookup::Lookup;
use crate::session_name::SessionName;
use crate::view::View;
use crate::vocabulary::rejection_name::RejectionName;
use crate::vocabulary::speller::Speller;
use crate::vocabulary::spelling::Spelling;
use crate::vocabulary::variant::Variant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputName {
    RecoveryStarted,
    RecoveryAdvanced,
    RecoveryCompleted,
    RecoveryCancelled,
    RecoveryUnaffected,
    RecoveryNotStarted,
    RecoveryConflicted,
    RecoveryRejected,
    RecoveriesListed,
    RecoveryShown,
    MissingChunksListed,
    RegistryShown,
}

impl OutputName {
    pub const SPELLING: Spelling<OutputName> = Spelling {
        kind: "an output",
        words: &[
            (OutputName::RecoveryStarted, "RecoveryStarted"),
            (OutputName::RecoveryAdvanced, "RecoveryAdvanced"),
            (OutputName::RecoveryCompleted, "RecoveryCompleted"),
            (OutputName::RecoveryCancelled, "RecoveryCancelled"),
            (OutputName::RecoveryUnaffected, "RecoveryUnaffected"),
            (OutputName::RecoveryNotStarted, "RecoveryNotStarted"),
            (OutputName::RecoveryConflicted, "RecoveryConflicted"),
            (OutputName::RecoveryRejected, "RecoveryRejected"),
            (OutputName::RecoveriesListed, "RecoveriesListed"),
            (OutputName::RecoveryShown, "RecoveryShown"),
            (OutputName::MissingChunksListed, "MissingChunksListed"),
            (OutputName::RegistryShown, "RegistryShown"),
        ],
    };

    // What the output carries (§5-§10), read from the state the command met,
    // or the one it left where the output says what the call made: the name a
    // start minted, the chunks a partial left held, the checkpoint after it.
    #[must_use]
    pub fn encoded(self, rejection: Option<RejectionName>, call: &Call) -> Value {
        let named = call.command.session_id;
        let payload = match self {
            OutputName::RecoveryStarted => json!({"session_id": View::session_name(named.unwrap_or(
                SessionName {
                    node: call.after.node,
                    incarnation: call.after.incarnation,
                    count: call.after.name_count,
                },
            ))}),
            OutputName::RecoveryAdvanced => {
                let session = Lookup::session(call.after, named);
                json!({
                    "held": View::number(session.chunk_set.held()),
                    "chunks": View::number(session.chunks),
                })
            }
            OutputName::RecoveryCompleted
            | OutputName::RecoveryUnaffected
            | OutputName::RecoveryNotStarted => Value::Object(Map::new()),
            OutputName::RecoveryCancelled => call.command.reason.clone().expect("a reason"),
            OutputName::RecoveryConflicted => {
                let session = Lookup::session(call.before, named);
                json!({
                    "chunks": View::number(session.chunks),
                    "sender": View::number(session.sender),
                    "receiver": View::number(session.receiver),
                })
            }
            OutputName::RecoveryRejected => rejection.expect("a rejection").encoded(call),
            OutputName::RecoveriesListed => json!({"sessions": View::names(call.before)}),
            OutputName::RecoveryShown => {
                let reading = Lookup::read(call.before, named).expect("a session to read");
                json!({
                    "record": View::record(reading.session),
                    "finished": reading.finished.map(View::number),
                })
            }
            OutputName::MissingChunksListed => {
                let reading = Lookup::read(call.before, named).expect("a session to read");
                let session = reading.session;
                json!({"missing": View::runs(&session.chunk_set.missing(session.chunks))})
            }
            OutputName::RegistryShown => View::checkpoint(call.after),
        };
        Variant::of(Speller::name(&Self::SPELLING, self), payload)
    }
}
