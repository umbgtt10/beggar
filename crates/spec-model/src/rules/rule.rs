// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::rules::change::Change;
use crate::rules::situation::Situation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    pub situation: Situation,
    pub change: Change,
}
