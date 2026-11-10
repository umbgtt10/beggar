// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#![no_std]
#![forbid(unsafe_code)]

//! A `no_std`, protocol-independent recovery registry.
//!
//! **Slices 1 to 4 of five are built:** an instance, created fresh or from a
//! checkpoint, with an observer told of every call; both ends of a transfer --
//! `StartRecovery` and `SendRecoveryPartial` on the side that holds the chunks,
//! `AdoptRecovery` and `ReceiveRecoveryPartial` on the other, each accounting
//! the session to completion; `CancelRecovery`, which ends a session for a
//! reason of the caller's own type, recorded and never read; and
//! `ShowRegistry`, the checkpoint -- conformant to version 0.5.0 of the
//! specification in the repository's `spec/`. Nothing should depend on it until the slices are done.
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
//! # Observer
//!
//! Every call is reported to the [`Observer`](observer::Observer) the instance
//! is constructed with: the command and its receipt, once, by the time
//! `process` returns. Wire it to a log, telemetry or a test; it answers
//! nothing, so beggar's answers do not depend on it.
//! [`NoOpObserver`](no_op_observer::NoOpObserver) is provided for the "just
//! drive it" case, as `faction` provides its own.
//!
//! # Why `no_std`
//!
//! The intended consumers run on embedded targets, so the mode is declared
//! here at the first version rather than discovered later. A session's size
//! follows the gaps in what has arrived, not its chunk count, and both terminal
//! outcomes evict it.

extern crate alloc;

pub mod beggar;
pub mod checkpoint;
pub mod chunk_set;
pub mod command;
pub mod fresh;
pub mod instance_id;
pub mod no_op_observer;
pub mod observer;
pub mod opening_rows;
pub mod origin;
pub mod output;
pub mod partial_rows;
pub mod receipt;
pub mod registry;
pub mod rejection;
pub mod session_id;
pub mod session_record;
pub mod side;
