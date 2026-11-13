// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::vocabulary::spelling::Spelling;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    StartRecovery,
    AdoptRecovery,
    SendRecoveryPartial,
    ReceiveRecoveryPartial,
    CancelRecovery,
    ListRecoveries,
    ShowRecovery,
    ListMissingChunks,
    ShowRegistry,
}

impl CommandName {
    pub const SPELLING: Spelling<CommandName> = Spelling {
        kind: "a command",
        words: &[
            (CommandName::StartRecovery, "StartRecovery"),
            (CommandName::AdoptRecovery, "AdoptRecovery"),
            (CommandName::SendRecoveryPartial, "SendRecoveryPartial"),
            (
                CommandName::ReceiveRecoveryPartial,
                "ReceiveRecoveryPartial",
            ),
            (CommandName::CancelRecovery, "CancelRecovery"),
            (CommandName::ListRecoveries, "ListRecoveries"),
            (CommandName::ShowRecovery, "ShowRecovery"),
            (CommandName::ListMissingChunks, "ListMissingChunks"),
            (CommandName::ShowRegistry, "ShowRegistry"),
        ],
    };
}
