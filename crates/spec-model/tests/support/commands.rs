// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_spec_model::command::Command;
use beggar_spec_model::run::Run;
use beggar_spec_model::session_name::SessionName;
use beggar_spec_model::vocabulary::command_name::CommandName;
use serde_json::json;

// Commands as the reader gives them, each carrying what its kind carries.
pub struct Commands;

impl Commands {
    pub fn adopt(name: SessionName, chunks: u64, sender: u64, receiver: u64) -> Command {
        Command {
            session_id: Some(name),
            chunks: Some(chunks),
            sender: Some(sender),
            receiver: Some(receiver),
            ..Self::of(CommandName::AdoptRecovery)
        }
    }

    pub fn cancel(name: SessionName) -> Command {
        Command {
            reason: Some(json!({"Superseded": {}})),
            ..Self::naming(CommandName::CancelRecovery, name)
        }
    }

    pub fn naming(command: CommandName, name: SessionName) -> Command {
        Command {
            session_id: Some(name),
            ..Self::of(command)
        }
    }

    pub fn of(name: CommandName) -> Command {
        Command {
            name,
            session_id: None,
            chunks: None,
            sender: None,
            receiver: None,
            run: None,
            reason: None,
        }
    }

    // A partial of the run, between node 1, sending, and node 2.
    pub fn partial(command: CommandName, name: SessionName, first: u64, last: u64) -> Command {
        Command {
            sender: Some(1),
            receiver: Some(2),
            run: Some(Run { first, last }),
            ..Self::naming(command, name)
        }
    }

    pub fn start(chunks: u64, sender: u64, receiver: u64) -> Command {
        Command {
            chunks: Some(chunks),
            sender: Some(sender),
            receiver: Some(receiver),
            ..Self::of(CommandName::StartRecovery)
        }
    }
}
