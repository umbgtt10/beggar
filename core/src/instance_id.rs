// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The instance id: a node's id and an incarnation the caller guarantees never
// repeats for that node. Every session the instance opens is named after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceId {
    pub node: u64,
    pub incarnation: u64,
}
