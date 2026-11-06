// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#![no_std]
#![forbid(unsafe_code)]

//! A `no_std`, protocol-independent recovery registry.
//!
//! **This version reserves the name. There is no implementation, so nothing
//! should depend on it yet.**
//!
//! # What it does
//!
//! A recovery is a collection of chunks -- a request for blocks, a lagging
//! follower's missing log entries, a snapshot sent in pieces -- and beggar
//! accounts for which of them have moved. It knows sessions, chunk counts,
//! chunk ids and node ids, and nothing else: what a chunk holds, in what order
//! chunks move, which protocol moves them and under what policy are the
//! caller's.
//!
//! # What the name means
//!
//! Recovery is a supplicant. A node that fell behind has no leverage over the
//! peers that did not: it asks for what it missed, the peer may refuse, it may
//! be handed something other than what it asked for, and if ignored its only
//! recourse is to ask again. Nothing in that loop is a negotiation, which is
//! why the crate is not called `sync` or `catchup`.
//!
//! # Sibling to `faction`
//!
//! [`faction`](https://crates.io/crates/faction) is the same idea applied to
//! the moment a cluster starts, and the two differ where it matters. Faction is
//! a state machine; beggar is a registry. Every outcome here is a function of
//! the command and the session's record, so no state label does any work and
//! there is no set of admissible commands to return. Serving needs no rule to
//! stay possible: the node that answers is the one already in sync, and a
//! registry refuses it nothing.
//!
//! # Why `no_std`
//!
//! The intended consumers run on embedded targets, so the mode is declared
//! here at the first version rather than discovered later. A session's size
//! follows the gaps in what has arrived, not its chunk count, and both terminal
//! outcomes evict it.

pub mod dummy;
