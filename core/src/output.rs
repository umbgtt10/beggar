// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::checkpoint::Checkpoint;
use crate::rejection::Rejection;
use crate::session_id::SessionId;

// What happened: past participle, as every output is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    RecoveryStarted { session_id: SessionId },
    RecoveryRejected(Rejection),
    RegistryShown(Checkpoint),
}
