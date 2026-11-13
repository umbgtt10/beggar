// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

//! The testing ladder's cluster simulation tier, laid out as faction's
//! `core-validation` is: a deterministic scenario harness and a cluster
//! simulation over a seeded faulty transport, both driving several beggar
//! instances as their control layers would, and the invariants a recovery
//! must keep across the cluster. Not published: it exists for its tests.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod cluster_invariants;
pub mod cluster_simulation;
pub mod scenario_event;
pub mod scenario_harness;
pub mod scenario_node;
pub mod scenario_reason;
pub mod scenario_trace_entry;
pub mod trace_observer;
pub mod transfer;
pub mod transport;
