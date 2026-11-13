// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance_id::InstanceId;
use crate::retained_record::RetainedRecord;
use crate::session_record::SessionRecord;
use alloc::vec::Vec;

// The whole registry, as ShowRegistry answers it and as an instance is
// restored from it: the instance id, the retention and the bound, every open
// and retained record in the order of their names, the count names are minted
// from, and the next receipt's number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    pub instance: InstanceId,
    pub retention: u64,
    pub bound: Option<u64>,
    pub open: Vec<SessionRecord>,
    pub retained: Vec<RetainedRecord>,
    pub name_count: u64,
    pub next_receipt: u64,
}
