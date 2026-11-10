// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Why a command was refused. A refusal changes nothing but the receipt number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    NoChunks,
    SelfAddressed,
    NotAParty,
    OwnSession,
    RegistryFull { capacity: u64 },
    EmptyRun { first: u64, last: u64 },
    Misattributed { sender: u64, receiver: u64 },
    WrongSide,
    ChunkOutOfRange { first: u64, last: u64, chunks: u64 },
}
