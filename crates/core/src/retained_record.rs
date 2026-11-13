// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::session_record::SessionRecord;

// A finished session the retention keeps: its record as it stood when it
// finished, and the number of the receipt that finished it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedRecord {
    pub record: SessionRecord,
    pub finished: u64,
}
