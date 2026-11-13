// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Why a scenario's stand-in control layer cancels a session, and the
// conformance vectors' reason: an enumeration of their own, standing for any
// caller's -- unservable, superseded by a larger session, the peer failed, or
// stalled with so many chunks held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioReason {
    Unservable,
    Superseded,
    PeerFailed,
    Stalled { held: u64 },
}
