// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;
use serde_json::Value;

use crate::chunk_set::ChunkSet;
use crate::retained::Retained;
use crate::session::Session;
use crate::session_name::SessionName;
use crate::value_reader::ValueReader;

// A checkpoint's parts read (§10): a session's name, its record -- the chunk
// set listed as ids -- and a retained record with the receipt that finished it.
pub struct RecordReader;

impl RecordReader {
    #[must_use]
    pub fn retained(value: &Value) -> Retained {
        Retained {
            session: Self::session(&value["record"]),
            finished: ValueReader::number(&value["finished"]),
        }
    }

    #[must_use]
    pub fn session(value: &Value) -> Session {
        let ids: Vec<u64> = ValueReader::list(&value["chunk_set"])
            .map(ValueReader::number)
            .collect();
        Session {
            name: Self::session_name(&value["session_id"]),
            chunks: ValueReader::number(&value["chunks"]),
            sender: ValueReader::number(&value["sender"]),
            receiver: ValueReader::number(&value["receiver"]),
            chunk_set: ChunkSet::of(&ids),
        }
    }

    #[must_use]
    pub fn session_name(value: &Value) -> SessionName {
        SessionName {
            node: ValueReader::number(&value["node"]),
            incarnation: ValueReader::number(&value["incarnation"]),
            count: ValueReader::number(&value["count"]),
        }
    }
}
