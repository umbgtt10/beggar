// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::states::States;
use beggar_spec_model::command::Command;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::rules::session_situation::SessionSituation;
use beggar_spec_model::vocabulary::command_name::CommandName;

fn holds(situation: SessionSituation, command: &Command) -> bool {
    situation.holds(command, &receiving())
}

fn receive(first: u64, last: u64) -> Command {
    Commands::partial(
        CommandName::ReceiveRecoveryPartial,
        States::peers(1),
        first,
        last,
    )
}

// The peer's session 1 open here, at its receiving end, holding chunks 1 and 3
// of its four.
fn receiving() -> InstanceState {
    States::holding(vec![States::session(States::peers(1), &[1, 3])], Vec::new())
}

#[test]
fn holds_of_advances_of_a_run_adding_a_chunk_short_of_the_whole_is_true() {
    // Arrange & Act & Assert
    assert!(holds(SessionSituation::Advances, &receive(2, 3)));
}

#[test]
fn holds_of_advances_of_a_run_already_held_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(SessionSituation::Advances, &receive(3, 3)));
}

#[test]
fn holds_of_advances_of_a_run_filling_the_set_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(SessionSituation::Advances, &receive(2, 4)));
}

#[test]
fn holds_of_alike_of_an_adopt_as_the_open_session_is_true() {
    // Arrange
    let command = Commands::adopt(States::peers(1), 4, 1, 2);

    // Act & Assert
    assert!(holds(SessionSituation::Alike, &command));
}

#[test]
fn holds_of_alike_of_an_adopt_with_another_count_is_false() {
    // Arrange
    let command = Commands::adopt(States::peers(1), 5, 1, 2);

    // Act & Assert
    assert!(!holds(SessionSituation::Alike, &command));
}

#[test]
fn holds_of_any_situation_with_no_session_of_the_name_open_is_false() {
    // Arrange
    let situations = [
        SessionSituation::Alike,
        SessionSituation::Differing,
        SessionSituation::Misattributed,
        SessionSituation::WrongSide,
        SessionSituation::OutOfRange,
        SessionSituation::Advances,
        SessionSituation::Completes,
    ];
    let command = Command {
        sender: Some(3),
        ..Commands::partial(CommandName::SendRecoveryPartial, States::peers(2), 0, 9)
    };

    // Act
    let held: Vec<bool> = situations
        .into_iter()
        .map(|situation| holds(situation, &command))
        .collect();

    // Assert
    assert_eq!(held, [false; 7]);
}

#[test]
fn holds_of_completes_of_a_run_already_held_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(SessionSituation::Completes, &receive(1, 1)));
}

#[test]
fn holds_of_completes_of_a_run_filling_the_set_is_true() {
    // Arrange & Act & Assert
    assert!(holds(SessionSituation::Completes, &receive(2, 4)));
}

#[test]
fn holds_of_differing_of_an_adopt_as_the_open_session_is_false() {
    // Arrange
    let command = Commands::adopt(States::peers(1), 4, 1, 2);

    // Act & Assert
    assert!(!holds(SessionSituation::Differing, &command));
}

#[test]
fn holds_of_differing_of_an_adopt_with_its_ends_swapped_is_true() {
    // Arrange
    let command = Commands::adopt(States::peers(1), 4, 2, 1);

    // Act & Assert
    assert!(holds(SessionSituation::Differing, &command));
}

#[test]
fn holds_of_misattributed_of_a_partial_naming_another_receiver_is_true() {
    // Arrange
    let command = Command {
        receiver: Some(3),
        ..receive(1, 1)
    };

    // Act & Assert
    assert!(holds(SessionSituation::Misattributed, &command));
}

#[test]
fn holds_of_misattributed_of_a_partial_naming_the_session_s_ends_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(SessionSituation::Misattributed, &receive(1, 1)));
}

#[test]
fn holds_of_out_of_range_of_a_run_from_0_or_past_the_count_is_true() {
    // Arrange
    let runs = [receive(0, 2), receive(4, 5)];

    // Act
    let held: Vec<bool> = runs
        .iter()
        .map(|command| holds(SessionSituation::OutOfRange, command))
        .collect();

    // Assert
    assert_eq!(held, [true, true]);
}

#[test]
fn holds_of_out_of_range_of_a_run_within_the_count_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(SessionSituation::OutOfRange, &receive(1, 4)));
}

#[test]
fn holds_of_wrong_side_of_a_receive_at_the_receiving_end_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(SessionSituation::WrongSide, &receive(1, 1)));
}

#[test]
fn holds_of_wrong_side_of_a_receive_at_the_sending_end_is_true() {
    // Arrange
    let state = InstanceState {
        node: 1,
        ..receiving()
    };

    // Act
    let held = SessionSituation::WrongSide.holds(&receive(1, 1), &state);

    // Assert
    assert!(held);
}

#[test]
fn holds_of_wrong_side_of_a_send_at_the_receiving_end_is_true() {
    // Arrange
    let command = Commands::partial(CommandName::SendRecoveryPartial, States::peers(1), 1, 1);

    // Act & Assert
    assert!(holds(SessionSituation::WrongSide, &command));
}
