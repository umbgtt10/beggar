// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_spec_model::vocabulary::command_name::CommandName;
use beggar_spec_model::vocabulary::speller::Speller;

const EVERY: [CommandName; 9] = [
    CommandName::StartRecovery,
    CommandName::AdoptRecovery,
    CommandName::SendRecoveryPartial,
    CommandName::ReceiveRecoveryPartial,
    CommandName::CancelRecovery,
    CommandName::ListRecoveries,
    CommandName::ShowRecovery,
    CommandName::ListMissingChunks,
    CommandName::ShowRegistry,
];

#[test]
fn name_of_each_command_is_the_vocabulary_s_in_the_commands_order() {
    // Arrange & Act
    let names: Vec<&str> = EVERY
        .iter()
        .map(|word| Speller::name(&CommandName::SPELLING, *word))
        .collect();

    // Assert
    assert_eq!(
        names,
        [
            "StartRecovery",
            "AdoptRecovery",
            "SendRecoveryPartial",
            "ReceiveRecoveryPartial",
            "CancelRecovery",
            "ListRecoveries",
            "ShowRecovery",
            "ListMissingChunks",
            "ShowRegistry",
        ]
    );
}

#[test]
#[should_panic(expected = "a command")]
fn of_a_name_no_command_has_panics() {
    // Arrange & Act & Assert
    let _ = Speller::of(&CommandName::SPELLING, "StopRecovery");
}

#[test]
fn of_each_command_s_name_reads_that_command() {
    // Arrange & Act
    let read: Vec<CommandName> = EVERY
        .iter()
        .map(|word| {
            Speller::of(
                &CommandName::SPELLING,
                Speller::name(&CommandName::SPELLING, *word),
            )
        })
        .collect();

    // Assert
    assert_eq!(read, EVERY);
}
