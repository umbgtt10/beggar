// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::receipt::Receipt;

// What an instance tells of every call, by the time it returns the receipt:
// the command and its receipt, the line a log holds. It is told and answers
// nothing, so it cannot change what the instance answers.
pub trait Observer {
    fn observe(&mut self, command: &Command, receipt: &Receipt);
}
