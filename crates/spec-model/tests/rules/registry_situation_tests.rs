// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::states::States;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::rules::registry_situation::RegistrySituation;
use beggar_spec_model::vocabulary::command_name::CommandName;

// Bound to two, the peer's sessions 1 and 2 open and its session 3 retained.
fn full() -> InstanceState {
    States::holding(
        vec![
            States::session(States::peers(1), &[]),
            States::session(States::peers(2), &[]),
        ],
        vec![States::retained(States::peers(3), 4)],
    )
}

fn holds(situation: RegistrySituation, count: u64, state: &InstanceState) -> bool {
    situation.holds(
        &Commands::naming(CommandName::AdoptRecovery, States::peers(count)),
        state,
    )
}

#[test]
fn holds_of_full_and_not_open_below_the_bound_is_false() {
    // Arrange
    let state = States::instance();

    // Act & Assert
    assert!(!holds(RegistrySituation::FullAndNotOpen, 3, &state));
}

#[test]
fn holds_of_full_and_not_open_of_a_name_not_open_when_full_is_true() {
    // Arrange & Act & Assert
    assert!(holds(RegistrySituation::FullAndNotOpen, 3, &full()));
}

#[test]
fn holds_of_full_and_not_open_of_a_name_open_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(RegistrySituation::FullAndNotOpen, 1, &full()));
}

#[test]
fn holds_of_full_below_the_bound_is_false() {
    // Arrange
    let state = States::holding(vec![States::session(States::peers(1), &[])], Vec::new());

    // Act & Assert
    assert!(!holds(RegistrySituation::Full, 1, &state));
}

#[test]
fn holds_of_full_with_the_open_sessions_numbering_the_bound_is_true() {
    // Arrange & Act & Assert
    assert!(holds(RegistrySituation::Full, 9, &full()));
}

#[test]
fn holds_of_neither_open_nor_retained_of_a_name_never_held_is_true() {
    // Arrange & Act & Assert
    assert!(holds(RegistrySituation::NeitherOpenNorRetained, 4, &full()));
}

#[test]
fn holds_of_neither_open_nor_retained_of_a_name_open_or_retained_is_false() {
    // Arrange
    let state = full();

    // Act
    let held: Vec<bool> = [1, 3]
        .into_iter()
        .map(|count| holds(RegistrySituation::NeitherOpenNorRetained, count, &state))
        .collect();

    // Assert
    assert_eq!(held, [false, false]);
}

#[test]
fn holds_of_not_open_of_a_name_only_retained_is_true() {
    // Arrange & Act & Assert
    assert!(holds(RegistrySituation::NotOpen, 3, &full()));
}

#[test]
fn holds_of_not_open_of_a_name_open_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(RegistrySituation::NotOpen, 2, &full()));
}

#[test]
fn holds_of_open_of_a_name_only_retained_is_false() {
    // Arrange & Act & Assert
    assert!(!holds(RegistrySituation::Open, 3, &full()));
}

#[test]
fn holds_of_open_of_a_name_open_is_true() {
    // Arrange & Act & Assert
    assert!(holds(RegistrySituation::Open, 2, &full()));
}
