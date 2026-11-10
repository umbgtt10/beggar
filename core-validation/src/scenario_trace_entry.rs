// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scenario_reason::ScenarioReason;
use beggar::command::Command;
use beggar::instance_id::InstanceId;
use beggar::receipt::Receipt;

// One call a node made, as its observer was told it: the instance that
// answered, the command and its receipt -- a line of the node's log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioTraceEntry {
    pub instance: InstanceId,
    pub command: Command<ScenarioReason>,
    pub receipt: Receipt<ScenarioReason>,
}
