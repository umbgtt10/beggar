// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::states::States;
use beggar_spec_model::rules::command_situation::CommandSituation;
use beggar_spec_model::rules::registry_situation::RegistrySituation;
use beggar_spec_model::rules::session_situation::SessionSituation;
use beggar_spec_model::rules::situation::Situation;
use beggar_spec_model::vocabulary::command_name::CommandName;

#[test]
fn holds_of_a_command_situation_is_that_situation_s() {
    // Arrange
    let situation = Situation::Command(CommandSituation::NoChunks);

    // Act
    let held = [Commands::start(0, 2, 1), Commands::start(1, 2, 1)]
        .map(|command| situation.holds(&command, &States::instance()));

    // Assert
    assert_eq!(held, [true, false]);
}

#[test]
fn holds_of_a_registry_situation_is_that_situation_s() {
    // Arrange
    let situation = Situation::Registry(RegistrySituation::Open);
    let state = States::holding(vec![States::session(States::peers(1), &[])], Vec::new());

    // Act
    let held = [States::peers(1), States::peers(2)]
        .map(|name| situation.holds(&Commands::naming(CommandName::CancelRecovery, name), &state));

    // Assert
    assert_eq!(held, [true, false]);
}

#[test]
fn holds_of_a_session_situation_is_that_situation_s() {
    // Arrange
    let situation = Situation::Session(SessionSituation::Alike);
    let state = States::holding(vec![States::session(States::peers(1), &[])], Vec::new());

    // Act
    let held = [
        Commands::adopt(States::peers(1), 4, 1, 2),
        Commands::adopt(States::peers(1), 3, 1, 2),
    ]
    .map(|command| situation.holds(&command, &state));

    // Assert
    assert_eq!(held, [true, false]);
}

#[test]
fn holds_of_always_is_true() {
    // Arrange
    let command = Commands::of(CommandName::ListRecoveries);

    // Act & Assert
    assert!(Situation::Always.holds(&command, &States::instance()));
}
