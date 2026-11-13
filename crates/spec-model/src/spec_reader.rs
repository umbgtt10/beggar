// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;
use serde_json::Value;

use crate::command::Command;
use crate::instance_state::InstanceState;
use crate::record_reader::RecordReader;
use crate::run::Run;
use crate::table::decision_table::DecisionTable;
use crate::table::row::Row;
use crate::value_reader::ValueReader;
use crate::vocabulary::command_name::CommandName;
use crate::vocabulary::output_name::OutputName;
use crate::vocabulary::rejection_name::RejectionName;
use crate::vocabulary::speller::Speller;

// The specification's encoding read: a command, an origin, the decision table.
// What it reads is valid against the schema, or outside the specification, and
// a reading of what is outside it panics.
pub struct SpecReader;

impl SpecReader {
    #[must_use]
    pub fn command(value: &Value) -> Command {
        let (name, payload) = value
            .as_object()
            .and_then(|object| object.iter().next())
            .expect("a command: an object of one member");
        Command {
            name: Speller::of(&CommandName::SPELLING, name),
            session_id: payload.get("session_id").map(RecordReader::session_name),
            chunks: payload.get("chunks").map(ValueReader::number),
            sender: payload.get("sender").map(ValueReader::number),
            receiver: payload.get("receiver").map(ValueReader::number),
            run: payload.get("first").map(|first| Run {
                first: ValueReader::number(first),
                last: ValueReader::number(&payload["last"]),
            }),
            reason: payload.get("reason").cloned(),
        }
    }

    // An instance as its origin makes it (§2): fresh, or exactly the state a
    // checkpoint describes.
    #[must_use]
    pub fn origin(value: &Value) -> InstanceState {
        value
            .get("Fresh")
            .map_or_else(|| Self::checkpoint(&value["Checkpoint"]), Self::fresh)
    }

    #[must_use]
    pub fn table(value: &Value) -> DecisionTable {
        DecisionTable::new(ValueReader::list(&value["rows"]).map(Self::row).collect())
    }

    fn checkpoint(value: &Value) -> InstanceState {
        InstanceState {
            open: ValueReader::list(&value["open"])
                .map(RecordReader::session)
                .collect(),
            retained: ValueReader::list(&value["retained"])
                .map(RecordReader::retained)
                .collect(),
            name_count: ValueReader::number(&value["name_count"]),
            next_receipt: ValueReader::number(&value["next_receipt"]),
            ..Self::fresh(value)
        }
    }

    // A fresh instance holds no session, has minted no name, and its first
    // receipt is number 1.
    fn fresh(value: &Value) -> InstanceState {
        InstanceState {
            node: ValueReader::number(&value["instance"]["node"]),
            incarnation: ValueReader::number(&value["instance"]["incarnation"]),
            retention: ValueReader::number(&value["retention"]),
            bound: (!value["bound"].is_null()).then(|| ValueReader::number(&value["bound"])),
            open: Vec::new(),
            retained: Vec::new(),
            name_count: 0,
            next_receipt: 1,
        }
    }

    fn row(value: &Value) -> Row {
        Row {
            id: String::from(ValueReader::string(&value["id"])),
            commands: ValueReader::list(&value["commands"])
                .map(|command| Speller::of(&CommandName::SPELLING, ValueReader::string(command)))
                .collect(),
            output: Speller::of(&OutputName::SPELLING, ValueReader::string(&value["output"])),
            rejection: value.get("rejection").map(|rejection| {
                Speller::of(&RejectionName::SPELLING, ValueReader::string(rejection))
            }),
        }
    }
}
