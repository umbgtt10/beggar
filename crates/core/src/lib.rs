// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#![no_std]
#![forbid(unsafe_code)]

//! **A `no_std`, zero-`unsafe`, protocol-independent recovery registry.**
//!
//! A recovery is a collection of chunks -- a request for blocks, a lagging
//! follower's missing log entries, a snapshot sent in pieces -- and beggar
//! accounts for which of them have moved, at both ends of a transfer. It knows
//! sessions, chunk counts, chunk ids and node ids, and nothing else: what a
//! chunk holds, in what order chunks move, which protocol moves them and under
//! what policy are the caller's.
//!
//! **All five slices are built** -- conformant to version 0.7.0 of the
//! specification in the repository's `spec/`.
//!
//! # What it does
//!
//! Both ends of a transfer: `StartRecovery` and `SendRecoveryPartial` on the
//! side that holds the chunks, `AdoptRecovery` and `ReceiveRecoveryPartial` on
//! the other, each accounting the session to completion; `CancelRecovery`, which
//! ends a session for a reason of the caller's own type, recorded and never read;
//! the queries -- `ListRecoveries`, `ShowRecovery`, `ListMissingChunks`, and
//! `ShowRegistry`, the checkpoint. An instance is created fresh, or from a
//! checkpoint for a replay.
//!
//! A finished session -- completed or cancelled -- is evicted; the instance's
//! **retention** keeps that many of them, the latest to finish, as they stood,
//! for the queries to read. It changes what a query answers and nothing else,
//! and it has no default: every instance states its own, 0 where nothing is to
//! be kept. An optional bound on open sessions refuses an open past it.
//!
//! Recovery is a supplicant -- a node that fell behind asks, the peer may
//! refuse, and if ignored its only recourse is to ask again -- which is why the
//! crate is not called `sync` or `catchup`.
//!
//! # Why beggar
//!
//! * **Deterministic** -- every output is a function of the command and the session's record
//! * **Specified** -- a normative specification with conformance vectors; this crate is one implementation of it
//! * **Replayable** -- every receipt is numbered by beggar, so a log rebuilds the registry and a gap proves a lost line
//! * **Zero unsafe** -- `#![forbid(unsafe_code)]`
//! * **`no_std + alloc`** -- a session's size follows the gaps in what has arrived, not its chunk count, so it runs on a microcontroller as on a server
//!
//! # Example
//!
//! ```rust
//! use beggar::beggar::Beggar;
//! use beggar::command::Command;
//! use beggar::fresh::Fresh;
//! use beggar::instance_id::InstanceId;
//! use beggar::no_op_observer::NoOpObserver;
//! use beggar::origin::Origin;
//! use beggar::output::Output;
//!
//! // The caller's own reasons for giving a transfer up: beggar records them and
//! // never reads them.
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum Reason {
//!     PeerFailed,
//! }
//!
//! fn fresh(node: u64) -> Beggar<Reason> {
//!     Beggar::new(
//!         Origin::Fresh(Fresh { instance: InstanceId { node, incarnation: 1 }, retention: 0, bound: None }),
//!         Box::new(NoOpObserver),
//!     )
//! }
//!
//! // Node 1 holds ten chunks node 2 lacks: it starts the session, and beggar names it.
//! let mut sender = fresh(1);
//! let Output::RecoveryStarted { session_id } = sender
//!     .process(Command::StartRecovery { chunks: 10, sender: 1, receiver: 2 })
//!     .output
//! else {
//!     unreachable!("a fresh registry starts a well-formed session")
//! };
//!
//! // Node 2 adopts the session under that name, with the count node 1 stated.
//! let mut receiver = fresh(2);
//! receiver.process(Command::AdoptRecovery { session_id, chunks: 10, sender: 1, receiver: 2 });
//!
//! // Chunks 1 to 6 move; each end records the run on its own side.
//! sender.process(Command::SendRecoveryPartial { session_id, first: 1, last: 6, sender: 1, receiver: 2 });
//! let receipt =
//!     receiver.process(Command::ReceiveRecoveryPartial { session_id, first: 1, last: 6, sender: 1, receiver: 2 });
//! assert_eq!(receipt.output, Output::RecoveryAdvanced { held: 6, chunks: 10 });
//!
//! // What node 2 still lacks, as maximal runs.
//! let missing = receiver.process(Command::ListMissingChunks { session_id }).output;
//! assert_eq!(missing, Output::MissingChunksListed { missing: vec![7..=10] });
//!
//! // The rest arrives, and the session completes.
//! let receipt =
//!     receiver.process(Command::ReceiveRecoveryPartial { session_id, first: 7, last: 10, sender: 1, receiver: 2 });
//! assert_eq!(receipt.output, Output::RecoveryCompleted);
//!
//! // Node 1 gives up on its end, for a reason of its own.
//! let receipt = sender.process(Command::CancelRecovery { session_id, reason: Reason::PeerFailed });
//! assert_eq!(receipt.output, Output::RecoveryCancelled(Reason::PeerFailed));
//! ```
//!
//! # Observer
//!
//! Every call is reported to the [`Observer`](observer::Observer) the instance
//! is constructed with: the command and its receipt, once, by the time
//! `process` returns. It answers nothing, so beggar's answers do not depend on
//! it. [`NoOpObserver`](no_op_observer::NoOpObserver) is provided for the "just
//! drive it" case.
//!
//! # Siblings
//!
//! Beggar is one of four bookkeepers a consensus protocol orchestrates, none
//! aware of the others: [`faction`](https://crates.io/crates/faction) keeps
//! membership, [`beggar`](https://crates.io/crates/beggar) recovery, `notary`
//! credentials (not yet built), and [`cartel`](https://crates.io/crates/cartel)
//! consensus. All are `no_std`, protocol-independent and specified, each with one
//! entry point and an observer.
//! Faction is a state machine; beggar, like cartel, is a registry: every outcome
//! is a function of the command and the session's record.
//!
//! # Further reading
//!
//! * [README](https://github.com/umbgtt10/beggar#readme) -- the overview: how it works, validation, project status
//! * [Specification](https://github.com/umbgtt10/beggar/blob/main/spec/SPEC.md) -- normative: this crate is one implementation of it
//! * [Design](https://github.com/umbgtt10/beggar/blob/main/docs/DESIGN.md) -- the design
//! * [ADRs](https://github.com/umbgtt10/beggar/tree/main/docs/ADRs) -- the decisions, one per file

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
pub mod query_rows;
pub mod receipt;
pub mod registry;
pub mod rejection;
pub mod retained_record;
pub mod session_id;
pub mod session_record;
pub mod side;
