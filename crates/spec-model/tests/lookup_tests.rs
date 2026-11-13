// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::states::States;
use beggar_spec_model::instance_state::InstanceState;
use beggar_spec_model::lookup::Lookup;
use beggar_spec_model::retained::Retained;

#[test]
fn is_full_below_the_bound_is_false() {
    // Arrange
    let state = States::holding(vec![States::session(States::peers(1), &[])], Vec::new());

    // Act
    let full = Lookup::is_full(&state);

    // Assert
    assert!(!full);
}

#[test]
fn is_full_with_no_bound_is_false() {
    // Arrange
    let state = InstanceState {
        bound: None,
        ..States::holding(vec![States::session(States::peers(1), &[])], Vec::new())
    };

    // Act
    let full = Lookup::is_full(&state);

    // Assert
    assert!(!full);
}

#[test]
fn is_full_with_open_sessions_numbering_the_bound_is_true() {
    // Arrange
    let state = States::holding(
        vec![
            States::session(States::peers(1), &[]),
            States::session(States::peers(2), &[]),
        ],
        Vec::new(),
    );

    // Act
    let full = Lookup::is_full(&state);

    // Assert
    assert!(full);
}

#[test]
fn open_of_a_name_not_open_is_none() {
    // Arrange
    let state = States::holding(
        vec![States::session(States::peers(1), &[])],
        vec![States::retained(States::peers(2), 4)],
    );

    // Act
    let open = Lookup::open(&state, Some(States::peers(2)));

    // Assert
    assert_eq!(open, None);
}

#[test]
fn open_of_a_name_open_is_that_session() {
    // Arrange
    let state = States::holding(
        vec![
            States::session(States::peers(1), &[]),
            States::session(States::peers(2), &[3]),
        ],
        Vec::new(),
    );

    // Act
    let open = Lookup::open(&state, Some(States::peers(2)));

    // Assert
    assert_eq!(open, Some(&States::session(States::peers(2), &[3])));
}

#[test]
fn open_of_no_name_is_none() {
    // Arrange
    let state = States::holding(vec![States::session(States::peers(1), &[])], Vec::new());

    // Act
    let open = Lookup::open(&state, None);

    // Assert
    assert_eq!(open, None);
}

#[test]
fn read_of_a_name_neither_open_nor_retained_is_none() {
    // Arrange
    let state = States::holding(
        vec![States::session(States::peers(1), &[])],
        vec![States::retained(States::peers(2), 4)],
    );

    // Act
    let read = Lookup::read(&state, Some(States::peers(3)));

    // Assert
    assert!(read.is_none());
}

#[test]
fn read_of_a_name_open_and_retained_is_the_open_session_unfinished() {
    // Arrange
    let state = States::holding(
        vec![States::session(States::peers(1), &[2])],
        vec![States::retained(States::peers(1), 4)],
    );

    // Act
    let read = Lookup::read(&state, Some(States::peers(1)))
        .map(|reading| (reading.session.clone(), reading.finished));

    // Assert
    assert_eq!(read, Some((States::session(States::peers(1), &[2]), None)));
}

#[test]
fn read_of_a_name_retained_twice_is_the_one_that_finished_last() {
    // Arrange
    let last = Retained {
        session: States::session(States::peers(1), &[1, 2, 3, 4]),
        finished: 9,
    };
    let state = States::holding(
        Vec::new(),
        vec![
            last.clone(),
            States::retained(States::peers(1), 4),
            States::retained(States::peers(2), 11),
        ],
    );

    // Act
    let read = Lookup::read(&state, Some(States::peers(1)))
        .map(|reading| (reading.session.clone(), reading.finished));

    // Assert
    assert_eq!(read, Some((last.session, Some(9))));
}

#[test]
#[should_panic(expected = "the open session the command names")]
fn session_of_a_name_only_retained_panics() {
    // Arrange
    let state = States::holding(Vec::new(), vec![States::retained(States::peers(1), 4)]);

    // Act & Assert
    let _ = Lookup::session(&state, Some(States::peers(1)));
}

#[test]
fn session_of_a_name_open_is_that_session() {
    // Arrange
    let state = States::holding(
        vec![
            States::session(States::peers(1), &[2]),
            States::session(States::peers(2), &[]),
        ],
        Vec::new(),
    );

    // Act
    let session = Lookup::session(&state, Some(States::peers(1)));

    // Assert
    assert_eq!(session, &States::session(States::peers(1), &[2]));
}
