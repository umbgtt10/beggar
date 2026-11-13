// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scenario_event::ScenarioEvent;

// The kind of an event, for a fault aimed at one kind alone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MessageKind {
    Request,
    #[default]
    Run,
    Abandon,
}

impl MessageKind {
    pub fn of(event: &ScenarioEvent) -> MessageKind {
        match event {
            ScenarioEvent::Request { .. } => MessageKind::Request,
            ScenarioEvent::Run { .. } => MessageKind::Run,
            ScenarioEvent::Abandon { .. } => MessageKind::Abandon,
        }
    }
}
