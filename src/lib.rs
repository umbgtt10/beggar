// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0
// SPDX-License-Identifier: Apache-2.0

#![no_std]

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
//! # Why `no_std`
//!
//! The intended consumers run on embedded targets with no allocator growth
//! budget, so the mode is declared here at version 0.0.0 rather than discovered
//! later. State must be bounded per episode.
