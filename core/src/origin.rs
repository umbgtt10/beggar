// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::checkpoint::Checkpoint;
use crate::fresh::Fresh;

// Where an instance comes from: fresh, or a checkpoint -- for a replay or a
// vector, never a live restart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Fresh(Fresh),
    Checkpoint(Checkpoint),
}
