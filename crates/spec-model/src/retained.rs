// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::session::Session;

// A finished session kept for the queries (§2): as it stood when it finished,
// with the number of the receipt that finished it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retained {
    pub session: Session,
    pub finished: u64,
}
