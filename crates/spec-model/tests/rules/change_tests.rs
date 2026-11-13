// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::commands::Commands;
use crate::support::states::States;
use beggar_spec_model::chunk_set::ChunkSet;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::retained::Retained;
use beggar_spec_model::rules::change::Change;
use beggar_spec_model::session::Session;
use beggar_spec_model::vocabulary::command_name::CommandName;

#[test]
fn applied_of_a_finish_at_a_retention_of_0_keeps_nothing() {
    // Arrange
    let state = InstanceState {
        retention: 0,
        ..States::holding(vec![States::session(States::peers(1), &[1])], Vec::new())
    };

    // Act
    let changed = Change::SessionCancelled.applied(&Commands::cancel(States::peers(1)), &state);

    // Assert
    assert_eq!(
        changed,
        InstanceState {
            open: Vec::new(),
            ..state
        }
    );
}

#[test]
fn applied_of_a_finish_beyond_the_retention_lets_the_earliest_finished_go() {
    // Arrange
    let state = InstanceState {
        retention: 2,
        ..States::holding(
            vec![States::session(States::peers(1), &[])],
            vec![
                States::retained(States::peers(8), 4),
                States::retained(States::peers(9), 2),
            ],
        )
    };

    // Act
    let changed = Change::SessionCancelled.applied(&Commands::cancel(States::peers(1)), &state);

    // Assert
    assert_eq!(
        changed.retained,
        [
            States::retained(States::peers(8), 4),
            Retained {
                session: States::session(States::peers(1), &[]),
                finished: 5,
            },
        ]
    );
}

#[test]
fn applied_of_a_run_added_adds_it_to_the_named_session_alone() {
    // Arrange
    let state = States::holding(
        vec![
            States::session(States::peers(1), &[1]),
            States::session(States::peers(2), &[1]),
        ],
        Vec::new(),
    );
    let command = Commands::partial(CommandName::ReceiveRecoveryPartial, States::peers(2), 2, 3);

    // Act
    let changed = Change::RunAdded.applied(&command, &state);

    // Assert
    assert_eq!(
        changed,
        States::holding(
            vec![
                States::session(States::peers(1), &[1]),
                States::session(States::peers(2), &[1, 2, 3]),
            ],
            Vec::new(),
        )
    );
}

#[test]
fn applied_of_a_session_adopted_opens_it_under_its_name_minting_nothing() {
    // Arrange
    let state = States::instance();

    // Act
    let changed =
        Change::SessionAdopted.applied(&Commands::adopt(States::peers(5), 4, 1, 2), &state);

    // Assert
    assert_eq!(
        changed,
        States::holding(vec![States::session(States::peers(5), &[])], Vec::new())
    );
}

#[test]
fn applied_of_a_session_cancelled_evicts_it_as_it_stood_and_retains_it() {
    // Arrange
    let state = States::holding(
        vec![
            States::session(States::peers(1), &[1, 3]),
            States::session(States::peers(2), &[]),
        ],
        Vec::new(),
    );

    // Act
    let changed = Change::SessionCancelled.applied(&Commands::cancel(States::peers(1)), &state);

    // Assert
    assert_eq!(
        changed,
        States::holding(
            vec![States::session(States::peers(2), &[])],
            vec![States::retained(States::peers(1), 5)],
        )
    );
}

#[test]
fn applied_of_a_session_completed_evicts_it_full_and_retains_it_with_this_call_s_number() {
    // Arrange
    let state = States::holding(vec![States::session(States::peers(1), &[1, 3])], Vec::new());
    let command = Commands::partial(CommandName::ReceiveRecoveryPartial, States::peers(1), 2, 4);

    // Act
    let changed = Change::SessionCompleted.applied(&command, &state);

    // Assert
    assert_eq!(
        changed,
        States::holding(
            Vec::new(),
            vec![Retained {
                session: States::session(States::peers(1), &[1, 2, 3, 4]),
                finished: 5,
            }],
        )
    );
}

#[test]
fn applied_of_a_session_started_mints_the_next_name_and_opens_it_empty() {
    // Arrange
    let state = States::instance();

    // Act
    let changed = Change::SessionStarted.applied(&Commands::start(4, 2, 1), &state);

    // Assert
    assert_eq!(
        changed,
        InstanceState {
            open: vec![Session {
                name: States::own(3),
                chunks: 4,
                sender: 2,
                receiver: 1,
                chunk_set: ChunkSet::default(),
            }],
            name_count: 3,
            ..state
        }
    );
}

#[test]
fn applied_of_nothing_leaves_the_state_as_it_is() {
    // Arrange
    let state = States::holding(
        vec![States::session(States::peers(1), &[1])],
        vec![States::retained(States::peers(2), 4)],
    );

    // Act
    let changed = Change::Nothing.applied(&Commands::start(4, 2, 1), &state);

    // Assert
    assert_eq!(changed, state);
}
