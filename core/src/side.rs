// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The side of a session a partial is reported on: a send on its sender, a
// receive on its receiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Sending,
    Receiving,
}
