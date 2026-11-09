// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Why a command was refused. A refusal changes nothing but the receipt number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    NoChunks,
    SelfAddressed,
    NotAParty,
    RegistryFull { capacity: u64 },
}
