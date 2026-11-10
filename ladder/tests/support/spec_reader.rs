// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::checkpoint::Checkpoint;
use beggar::chunk_set::ChunkSet;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::origin::Origin;
use beggar::retained_record::RetainedRecord;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use beggar_core_validation::scenario_reason::ScenarioReason;
use serde_json::Value;

// The specification's JSON read into beggar's types, by the data model's own
// names: the crate knows nothing of JSON. A form 0.6.0 does not have is a
// broken vector, and panics saying which.
pub struct SpecReader;

impl SpecReader {
    pub fn checkpoint(value: &Value) -> Checkpoint {
        Checkpoint {
            instance: Self::instance(&value["instance"]),
            retention: Self::number(&value["retention"]),
            bound: Self::bound(&value["bound"]),
            open: Self::records(&value["open"]),
            retained: Self::retained(&value["retained"]),
            name_count: Self::number(&value["name_count"]),
            next_receipt: Self::number(&value["next_receipt"]),
        }
    }

    pub fn command(value: &Value) -> Command<ScenarioReason> {
        let (name, carried) = Self::member(value);
        match name {
            "StartRecovery" => Command::StartRecovery {
                chunks: Self::number(&carried["chunks"]),
                sender: Self::number(&carried["sender"]),
                receiver: Self::number(&carried["receiver"]),
            },
            "AdoptRecovery" => Command::AdoptRecovery {
                session_id: Self::session_id(&carried["session_id"]),
                chunks: Self::number(&carried["chunks"]),
                sender: Self::number(&carried["sender"]),
                receiver: Self::number(&carried["receiver"]),
            },
            "ReceiveRecoveryPartial" => Command::ReceiveRecoveryPartial {
                session_id: Self::session_id(&carried["session_id"]),
                first: Self::number(&carried["first"]),
                last: Self::number(&carried["last"]),
                sender: Self::number(&carried["sender"]),
                receiver: Self::number(&carried["receiver"]),
            },
            "SendRecoveryPartial" => Command::SendRecoveryPartial {
                session_id: Self::session_id(&carried["session_id"]),
                first: Self::number(&carried["first"]),
                last: Self::number(&carried["last"]),
                sender: Self::number(&carried["sender"]),
                receiver: Self::number(&carried["receiver"]),
            },
            "CancelRecovery" => Command::CancelRecovery {
                session_id: Self::session_id(&carried["session_id"]),
                reason: Self::reason(&carried["reason"]),
            },
            "ListRecoveries" => Command::ListRecoveries,
            "ShowRecovery" => Command::ShowRecovery {
                session_id: Self::session_id(&carried["session_id"]),
            },
            "ListMissingChunks" => Command::ListMissingChunks {
                session_id: Self::session_id(&carried["session_id"]),
            },
            "ShowRegistry" => Command::ShowRegistry,
            other => panic!("not a command of 0.6.0: {other}"),
        }
    }

    pub fn origin(value: &Value) -> Origin {
        let (name, carried) = Self::member(value);
        match name {
            "Fresh" => Origin::Fresh(Fresh {
                instance: Self::instance(&carried["instance"]),
                retention: Self::number(&carried["retention"]),
                bound: Self::bound(&carried["bound"]),
            }),
            "Checkpoint" => Origin::Checkpoint(Self::checkpoint(carried)),
            other => panic!("not an origin: {other}"),
        }
    }

    fn bound(value: &Value) -> Option<u64> {
        if value.is_null() {
            None
        } else {
            Some(Self::number(value))
        }
    }

    // A chunk set's ids, each added as a run of one: the set merges them.
    fn chunk_set(value: &Value) -> ChunkSet {
        let mut chunk_set = ChunkSet::new();
        for id in value
            .as_array()
            .unwrap_or_else(|| panic!("a chunk set is a list of ids: {value}"))
        {
            let id = Self::number(id);
            chunk_set.add(id, id);
        }
        chunk_set
    }

    fn instance(value: &Value) -> InstanceId {
        InstanceId {
            node: Self::number(&value["node"]),
            incarnation: Self::number(&value["incarnation"]),
        }
    }

    fn member(value: &Value) -> (&str, &Value) {
        let object = value
            .as_object()
            .unwrap_or_else(|| panic!("an enumeration's value is an object: {value}"));
        let (name, carried) = object
            .iter()
            .next()
            .unwrap_or_else(|| panic!("of one member: {value}"));
        (name.as_str(), carried)
    }

    fn number(value: &Value) -> u64 {
        value
            .as_str()
            .and_then(|digits| digits.parse().ok())
            .unwrap_or_else(|| panic!("a u64 is a string of decimal digits: {value}"))
    }

    // The vectors' own reason, read into the scenarios' reason type.
    fn reason(value: &Value) -> ScenarioReason {
        let (name, carried) = Self::member(value);
        match name {
            "Unservable" => ScenarioReason::Unservable,
            "Superseded" => ScenarioReason::Superseded,
            "PeerFailed" => ScenarioReason::PeerFailed,
            "Stalled" => ScenarioReason::Stalled {
                held: Self::number(&carried["held"]),
            },
            other => panic!("not a reason of 0.6.0: {other}"),
        }
    }

    fn record(value: &Value) -> SessionRecord {
        SessionRecord {
            session_id: Self::session_id(&value["session_id"]),
            chunks: Self::number(&value["chunks"]),
            sender: Self::number(&value["sender"]),
            receiver: Self::number(&value["receiver"]),
            chunk_set: Self::chunk_set(&value["chunk_set"]),
        }
    }

    fn records(value: &Value) -> Vec<SessionRecord> {
        value
            .as_array()
            .unwrap_or_else(|| panic!("records are a list: {value}"))
            .iter()
            .map(Self::record)
            .collect()
    }

    fn retained(value: &Value) -> Vec<RetainedRecord> {
        value
            .as_array()
            .unwrap_or_else(|| panic!("retained records are a list: {value}"))
            .iter()
            .map(|kept| RetainedRecord {
                record: Self::record(&kept["record"]),
                finished: Self::number(&kept["finished"]),
            })
            .collect()
    }

    fn session_id(value: &Value) -> SessionId {
        SessionId {
            node: Self::number(&value["node"]),
            incarnation: Self::number(&value["incarnation"]),
            count: Self::number(&value["count"]),
        }
    }
}
