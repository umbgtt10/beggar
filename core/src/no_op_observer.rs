// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::observer::Observer;
use crate::receipt::Receipt;

// The observer for an instance with no use for one: told of every call, it
// does nothing.
pub struct NoOpObserver;

impl Observer for NoOpObserver {
    fn observe(&mut self, _command: &Command, _receipt: &Receipt) {}
}
