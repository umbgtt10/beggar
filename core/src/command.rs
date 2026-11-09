// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What a caller asks of an instance: imperative, as every command is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    StartRecovery {
        chunks: u64,
        sender: u64,
        receiver: u64,
    },
    ShowRegistry,
}
