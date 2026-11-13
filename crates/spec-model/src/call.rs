// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance_state::InstanceState;

// One call as its output is written from it: the command, and the instance's
// state before and after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Call<'a> {
    pub command: &'a Command,
    pub before: &'a InstanceState,
    pub after: &'a InstanceState,
}
