// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::origin::Origin;
use beggar::received_chunks::ReceivedChunks;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use serde_json::Value;

// The specification's JSON read into beggar's types, by the data model's own
// names: the crate knows nothing of JSON. A form 0.1.0 does not have is a
// broken vector, and panics saying which.
pub struct SpecReader;

impl SpecReader {
    pub fn checkpoint(value: &Value) -> Checkpoint {
        Checkpoint {
            instance: Self::instance(&value["instance"]),
            retention: Self::number(&value["retention"]),
            bound: Self::bound(&value["bound"]),
            open: Self::records(&value["open"]),
            retained: Self::records(&value["retained"]),
            name_count: Self::number(&value["name_count"]),
            next_receipt: Self::number(&value["next_receipt"]),
        }
    }

    pub fn command(value: &Value) -> Command {
        let (name, carried) = Self::member(value);
        match name {
            "StartRecovery" => Command::StartRecovery {
                chunks: Self::number(&carried["chunks"]),
                sender: Self::number(&carried["sender"]),
                receiver: Self::number(&carried["receiver"]),
            },
            "ShowRegistry" => Command::ShowRegistry,
            other => panic!("not a command of 0.1.0: {other}"),
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

    // 0.1.0 records no chunk, so every record's set is empty.
    fn received(value: &Value) -> ReceivedChunks {
        let ids = value
            .as_array()
            .unwrap_or_else(|| panic!("the chunks received are a set of ids: {value}"));
        assert!(ids.is_empty(), "0.1.0 receives no chunk: {value}");
        ReceivedChunks::new()
    }

    fn record(value: &Value) -> SessionRecord {
        SessionRecord {
            session_id: Self::session_id(&value["session_id"]),
            chunks: Self::number(&value["chunks"]),
            sender: Self::number(&value["sender"]),
            receiver: Self::number(&value["receiver"]),
            received: Self::received(&value["received"]),
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

    fn session_id(value: &Value) -> SessionId {
        SessionId {
            node: Self::number(&value["node"]),
            incarnation: Self::number(&value["incarnation"]),
            count: Self::number(&value["count"]),
        }
    }
}
