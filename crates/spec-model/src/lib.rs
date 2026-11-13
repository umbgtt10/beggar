// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

//! # beggar-spec-model
//!
//! **An independent model of beggar's specification.**
//!
//! The specification in `spec/` is normative, and the `beggar` crate is one
//! implementation of it. This is a second one, written from the specification's
//! prose and decision table alone, and kept apart from the crate: it depends on
//! nothing of `beggar`, and the two meet only in the specification's own
//! encoding, in the ladder. Its vectors are checked against it before the crate
//! runs them, and the crate is checked against it over command sequences no
//! vector holds.
//!
//! Its shape is the specification's rule, not the crate's registry: a command is
//! answered by the first row of the decision table that applies, in the table's
//! order. The model reads the rows, and owns only what the prose gives each one
//! -- the situation in which it applies and the change it makes -- what its
//! output carries, and the checkpoint.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod answer;
pub mod call;
pub mod chunk_set;
pub mod command;
pub mod instance_state;
pub mod lookup;
pub mod model;
pub mod reading;
pub mod record_reader;
pub mod retained;
pub mod rules;
pub mod run;
pub mod session;
pub mod session_name;
pub mod spec_reader;
pub mod table;
pub mod value_reader;
pub mod view;
pub mod vocabulary;
