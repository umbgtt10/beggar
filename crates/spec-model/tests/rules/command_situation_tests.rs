// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::states::States;
use beggar_spec_model::command::Command;
use beggar_spec_model::rules::command_situation::CommandSituation;
use beggar_spec_model::session_name::SessionName;
use beggar_spec_model::vocabulary::command_name::CommandName;

fn holds(situation: CommandSituation, command: &Command) -> bool {
    situation.holds(command, &States::instance())
}

#[test]
fn holds_of_an_empty_run_with_a_run_of_one_is_false() {
    // Arrange
    let command = Commands::partial(CommandName::SendRecoveryPartial, States::peers(1), 3, 3);

    // Act & Assert
    assert!(!holds(CommandSituation::EmptyRun, &command));
}

#[test]
fn holds_of_an_empty_run_with_its_last_below_its_first_is_true() {
    // Arrange
    let command = Commands::partial(CommandName::SendRecoveryPartial, States::peers(1), 4, 3);

    // Act & Assert
    assert!(holds(CommandSituation::EmptyRun, &command));
}

#[test]
fn holds_of_no_chunks_with_a_count_above_0_is_false() {
    // Arrange
    let command = Commands::start(1, 2, 1);

    // Act & Assert
    assert!(!holds(CommandSituation::NoChunks, &command));
}

#[test]
fn holds_of_no_chunks_with_a_count_of_0_is_true() {
    // Arrange
    let command = Commands::start(0, 2, 1);

    // Act & Assert
    assert!(holds(CommandSituation::NoChunks, &command));
}

#[test]
fn holds_of_not_a_party_with_this_node_at_either_end_is_false() {
    // Arrange
    let commands = [Commands::start(4, 2, 1), Commands::start(4, 1, 2)];

    // Act
    let held: Vec<bool> = commands
        .iter()
        .map(|command| holds(CommandSituation::NotAParty, command))
        .collect();

    // Assert
    assert_eq!(held, [false, false]);
}

#[test]
fn holds_of_not_a_party_with_this_node_at_neither_end_is_true() {
    // Arrange
    let command = Commands::start(4, 1, 3);

    // Act & Assert
    assert!(holds(CommandSituation::NotAParty, &command));
}

#[test]
fn holds_of_own_session_of_a_name_another_incarnation_minted_is_false() {
    // Arrange
    let command = Commands::adopt(
        SessionName {
            node: 2,
            incarnation: 4,
            count: 1,
        },
        4,
        1,
        2,
    );

    // Act & Assert
    assert!(!holds(CommandSituation::OwnSession, &command));
}

#[test]
fn holds_of_own_session_of_a_name_this_instance_minted_is_true() {
    // Arrange
    let command = Commands::adopt(States::own(9), 4, 1, 2);

    // Act & Assert
    assert!(holds(CommandSituation::OwnSession, &command));
}

#[test]
fn holds_of_self_addressed_with_one_node_at_both_ends_is_true() {
    // Arrange
    let command = Commands::start(4, 2, 2);

    // Act & Assert
    assert!(holds(CommandSituation::SelfAddressed, &command));
}

#[test]
fn holds_of_self_addressed_with_two_nodes_is_false() {
    // Arrange
    let command = Commands::start(4, 2, 1);

    // Act & Assert
    assert!(!holds(CommandSituation::SelfAddressed, &command));
}
