// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::instance_id::InstanceId;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use beggar_core_validation::scenario_reason::ScenarioReason;
use serde_json::json;
use serde_json::Value;

// Beggar's answers written in the specification's JSON, so a receipt compares
// with a vector's as the specification states it.
pub struct SpecWriter;

impl SpecWriter {
    pub fn checkpoint(checkpoint: &Checkpoint) -> Value {
        json!({
            "instance": Self::instance(&checkpoint.instance),
            "retention": Self::number(checkpoint.retention),
            "bound": checkpoint.bound.map_or(Value::Null, Self::number),
            "open": Self::records(&checkpoint.open),
            "retained": Self::records(&checkpoint.retained),
            "name_count": Self::number(checkpoint.name_count),
            "next_receipt": Self::number(checkpoint.next_receipt),
        })
    }

    pub fn command(command: &Command<ScenarioReason>) -> Value {
        match command {
            Command::StartRecovery {
                chunks,
                sender,
                receiver,
            } => json!({ "StartRecovery": {
                "chunks": Self::number(*chunks),
                "sender": Self::number(*sender),
                "receiver": Self::number(*receiver),
            } }),
            Command::AdoptRecovery {
                session_id,
                chunks,
                sender,
                receiver,
            } => json!({ "AdoptRecovery": {
                "session_id": Self::session_id(session_id),
                "chunks": Self::number(*chunks),
                "sender": Self::number(*sender),
                "receiver": Self::number(*receiver),
            } }),
            Command::ReceiveRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            } => json!({ "ReceiveRecoveryPartial": {
                "session_id": Self::session_id(session_id),
                "first": Self::number(*first),
                "last": Self::number(*last),
                "sender": Self::number(*sender),
                "receiver": Self::number(*receiver),
            } }),
            Command::SendRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            } => json!({ "SendRecoveryPartial": {
                "session_id": Self::session_id(session_id),
                "first": Self::number(*first),
                "last": Self::number(*last),
                "sender": Self::number(*sender),
                "receiver": Self::number(*receiver),
            } }),
            Command::CancelRecovery { session_id, reason } => json!({ "CancelRecovery": {
                "session_id": Self::session_id(session_id),
                "reason": Self::reason(reason),
            } }),
            Command::ShowRegistry => json!({ "ShowRegistry": {} }),
        }
    }

    pub fn receipt(receipt: &Receipt<ScenarioReason>) -> Value {
        json!({
            "number": Self::number(receipt.number),
            "output": Self::output(&receipt.output),
        })
    }

    // A log's line: the call's command and the receipt it was answered with.
    pub fn step(command: &Command<ScenarioReason>, receipt: &Receipt<ScenarioReason>) -> Value {
        json!({
            "command": Self::command(command),
            "receipt": Self::receipt(receipt),
        })
    }

    fn instance(instance: &InstanceId) -> Value {
        json!({
            "node": Self::number(instance.node),
            "incarnation": Self::number(instance.incarnation),
        })
    }

    fn number(value: u64) -> Value {
        Value::String(value.to_string())
    }

    fn output(output: &Output<ScenarioReason>) -> Value {
        match output {
            Output::RecoveryStarted { session_id } => {
                json!({ "RecoveryStarted": { "session_id": Self::session_id(session_id) } })
            }
            Output::RecoveryAdvanced { held, chunks } => {
                json!({ "RecoveryAdvanced": { "held": Self::number(*held), "chunks": Self::number(*chunks) } })
            }
            Output::RecoveryCompleted => json!({ "RecoveryCompleted": {} }),
            Output::RecoveryCancelled(reason) => {
                json!({ "RecoveryCancelled": Self::reason(reason) })
            }
            Output::RecoveryUnaffected => json!({ "RecoveryUnaffected": {} }),
            Output::RecoveryNotStarted => json!({ "RecoveryNotStarted": {} }),
            Output::RecoveryConflicted {
                chunks,
                sender,
                receiver,
            } => json!({ "RecoveryConflicted": {
                "chunks": Self::number(*chunks),
                "sender": Self::number(*sender),
                "receiver": Self::number(*receiver),
            } }),
            Output::RecoveryRejected(rejection) => {
                json!({ "RecoveryRejected": Self::rejection(rejection) })
            }
            Output::RegistryShown(checkpoint) => {
                json!({ "RegistryShown": Self::checkpoint(checkpoint) })
            }
        }
    }

    fn reason(reason: &ScenarioReason) -> Value {
        match reason {
            ScenarioReason::Unservable => json!({ "Unservable": {} }),
            ScenarioReason::Superseded => json!({ "Superseded": {} }),
            ScenarioReason::PeerFailed => json!({ "PeerFailed": {} }),
            ScenarioReason::Stalled { held } => {
                json!({ "Stalled": { "held": Self::number(*held) } })
            }
        }
    }

    fn records(records: &[SessionRecord]) -> Value {
        Value::Array(
            records
                .iter()
                .map(|record| {
                    json!({
                        "session_id": Self::session_id(&record.session_id),
                        "chunks": Self::number(record.chunks),
                        "sender": Self::number(record.sender),
                        "receiver": Self::number(record.receiver),
                        "chunk_set": record.chunk_set.ids().into_iter().map(Self::number).collect::<Vec<Value>>(),
                    })
                })
                .collect(),
        )
    }

    fn rejection(rejection: &Rejection) -> Value {
        match rejection {
            Rejection::NoChunks => json!({ "NoChunks": {} }),
            Rejection::SelfAddressed => json!({ "SelfAddressed": {} }),
            Rejection::NotAParty => json!({ "NotAParty": {} }),
            Rejection::OwnSession => json!({ "OwnSession": {} }),
            Rejection::RegistryFull { capacity } => {
                json!({ "RegistryFull": { "capacity": Self::number(*capacity) } })
            }
            Rejection::EmptyRun { first, last } => {
                json!({ "EmptyRun": { "first": Self::number(*first), "last": Self::number(*last) } })
            }
            Rejection::Misattributed { sender, receiver } => {
                json!({ "Misattributed": { "sender": Self::number(*sender), "receiver": Self::number(*receiver) } })
            }
            Rejection::WrongSide => json!({ "WrongSide": {} }),
            Rejection::ChunkOutOfRange {
                first,
                last,
                chunks,
            } => json!({ "ChunkOutOfRange": {
                "first": Self::number(*first),
                "last": Self::number(*last),
                "chunks": Self::number(*chunks),
            } }),
        }
    }

    fn session_id(session_id: &SessionId) -> Value {
        json!({
            "node": Self::number(session_id.node),
            "incarnation": Self::number(session_id.incarnation),
            "count": Self::number(session_id.count),
        })
    }
}
