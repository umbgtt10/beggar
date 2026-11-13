// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::spec_files::SpecFiles;
use crate::support::states::States;
use beggar_spec_model::command::Command;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::spec_reader::SpecReader;
use beggar_spec_model::table::decision_table::DecisionTable;
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

fn row_id(command: &Command, state: &InstanceState) -> String {
    SpecReader::table(&SpecFiles::matrix())
        .row(command, state)
        .id
        .clone()
}

#[test]
fn row_of_a_command_two_rows_apply_to_is_the_first_of_them() {
    // Arrange & Act
    let row = row_id(&Commands::start(4, 2, 2), &full());

    // Assert
    assert_eq!(row, "self-addressed");
}

#[test]
fn row_of_a_partial_of_a_name_only_retained_is_partial_not_started() {
    // Arrange
    let command = Commands::partial(CommandName::ReceiveRecoveryPartial, States::peers(3), 1, 1);

    // Act
    let row = row_id(&command, &full());

    // Assert
    assert_eq!(row, "partial-not-started");
}

#[test]
fn row_of_a_query_of_a_name_only_retained_is_recovery_shown() {
    // Arrange
    let command = Commands::naming(CommandName::ShowRecovery, States::peers(3));

    // Act
    let row = row_id(&command, &full());

    // Assert
    assert_eq!(row, "recovery-shown");
}

#[test]
fn row_of_a_repeated_adopt_when_full_is_adopt_unaffected() {
    // Arrange & Act
    let row = row_id(&Commands::adopt(States::peers(1), 4, 1, 2), &full());

    // Assert
    assert_eq!(row, "adopt-unaffected");
}

#[test]
fn row_of_a_start_below_the_bound_is_start_started() {
    // Arrange & Act
    let row = row_id(&Commands::start(4, 2, 1), &States::instance());

    // Assert
    assert_eq!(row, "start-started");
}

#[test]
#[should_panic(expected = "a row that applies")]
fn row_of_a_table_with_no_row_that_applies_panics() {
    // Arrange
    let table = DecisionTable::new(Vec::new());

    // Act & Assert
    let _ = table.row(
        &Commands::of(CommandName::ShowRegistry),
        &States::instance(),
    );
}
