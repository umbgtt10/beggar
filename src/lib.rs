// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#![no_std]
#![forbid(unsafe_code)]

//! Protocol-agnostic recovery for replicated logs.
//!
//! **This version reserves the name. There is no implementation, so nothing
//! should depend on it yet.**
//!
//! # What the name means
//!
//! Recovery is a supplicant. A node that fell behind has no leverage over the
//! peers that did not: it asks for the entries it missed, the peer may refuse,
//! it may be handed a snapshot instead of what it asked for, and if ignored its
//! only recourse is to ask again. Nothing in that loop is a negotiation, which
//! is why the crate is not called `sync` or `catchup`.
//!
//! # Sibling to `faction`
//!
//! [`faction`](https://crates.io/crates/faction) is the same idea applied to
//! the moment a cluster starts: a deterministic state machine the host drives,
//! with one entry point, a read-only probe, and the set of admissible commands
//! returned on every result. This crate is intended to follow that shape for
//! the moment a node falls behind.
//!
//! Two things differ, and both change the design. Faction converges once —
//! `Bootstrapped` is terminal. Recovery is cyclic, re-entered at every
//! partition, so episodes need an explicit counter where faction needed none.
//! And serving must stay admissible in *every* state, because the node that
//! answers a request is precisely the one that is already in sync.
//!
//! # Why `no_std`
//!
//! The intended consumers run on embedded targets with no allocator growth
//! budget, so the mode is declared here at the first version rather than
//! discovered later. State must be bounded per episode.
