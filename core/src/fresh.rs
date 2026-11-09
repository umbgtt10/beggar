// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance_id::InstanceId;

// A fresh instance's origin: its instance id, its retention, and its bound on
// open sessions, or none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fresh {
    pub instance: InstanceId,
    pub retention: u64,
    pub bound: Option<u64>,
}
