// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeSet;
use beggar::session_id::SessionId;

// What the cluster knows of one transfer from outside its nodes: its name, its
// ends and its count; the chunks its sender reported sent; the chunks
// delivered to its receiver, ever and since its session last opened there,
// and whether it is open there now; whether the sender completed it; how often
// the receiver completed and reopened it, and how many runs reached the
// receiver after the session first finished there, completed or cancelled;
// how many receipts, at either end, were not what those chunks called for;
// whether a restart interrupted it; how often the cancellation policy ended
// it at its receiver, and whether its sender cancelled the pages it still held
// on hearing so; and the step a run last reached its receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    pub session_id: SessionId,
    pub sender: u64,
    pub receiver: u64,
    pub chunks: u64,
    pub sent: BTreeSet<u64>,
    pub delivered: BTreeSet<u64>,
    pub held: BTreeSet<u64>,
    pub receiver_open: bool,
    pub sender_completed: bool,
    pub receiver_completions: u64,
    pub reopens: u64,
    pub late_copies: u64,
    pub unsound: u64,
    pub interrupted: bool,
    pub cancellations: u64,
    pub last_delivery: u64,
    pub sender_cancelled: bool,
}
