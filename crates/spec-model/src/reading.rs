// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::session::Session;

// The session a query reads (§9): the open one of its name, unfinished, or the
// retained one that finished last, with the number of the receipt that
// finished it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading<'a> {
    pub session: &'a Session,
    pub finished: Option<u64>,
}
