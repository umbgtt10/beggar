// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

//! The testing ladder's memory tier: a counting allocator, measured where the
//! allocator sees every `Layout` rather than where the registry believes it
//! is, and a run of recoveries one after another that it measures. Not
//! published: it exists for its tests.

pub mod allocation_counter;
pub mod counting_allocator;
pub mod memory_profile;
pub mod recovery_sequence;
