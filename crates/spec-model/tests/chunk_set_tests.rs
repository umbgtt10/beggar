// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_spec_model::chunk_set::ChunkSet;
use beggar_spec_model::run::Run;

const TOP: u64 = u64::MAX;

#[test]
fn held_of_a_run_at_the_top_of_u64_counts_each_chunk() {
    // Arrange
    let chunk_set = ChunkSet::default().with(Run {
        first: TOP - 2,
        last: TOP,
    });

    // Act
    let held = chunk_set.held();

    // Assert
    assert_eq!(held, 3);
}

#[test]
fn held_of_the_empty_set_is_0() {
    // Arrange & Act
    let held = ChunkSet::default().held();

    // Assert
    assert_eq!(held, 0);
}

#[test]
fn holds_of_a_run_inside_one_held_run_is_true() {
    // Arrange
    let chunk_set = ChunkSet::of(&[2, 3, 4, 7]);

    // Act
    let holds = chunk_set.holds(Run { first: 3, last: 4 });

    // Assert
    assert!(holds);
}

#[test]
fn holds_of_a_run_reaching_a_gap_is_false() {
    // Arrange
    let chunk_set = ChunkSet::of(&[2, 3, 4, 7]);

    // Act
    let holds = chunk_set.holds(Run { first: 4, last: 5 });

    // Assert
    assert!(!holds);
}

#[test]
fn ids_of_a_set_built_from_ids_in_any_order_are_each_once_ascending() {
    // Arrange
    let chunk_set = ChunkSet::of(&[9, 1, 3, 2, 3]);

    // Act
    let ids = chunk_set.ids();

    // Assert
    assert_eq!(ids, [1, 2, 3, 9]);
}

#[test]
fn missing_of_a_full_set_is_empty() {
    // Arrange
    let chunk_set = ChunkSet::of(&[1, 2, 3, 4]);

    // Act
    let missing = chunk_set.missing(4);

    // Assert
    assert!(missing.is_empty(), "{missing:?}");
}

#[test]
fn missing_of_a_set_holding_the_top_of_u64_lists_nothing_past_it() {
    // Arrange
    let chunk_set = ChunkSet::default().with(Run {
        first: TOP - 1,
        last: TOP,
    });

    // Act
    let missing = chunk_set.missing(TOP);

    // Assert
    assert_eq!(
        missing,
        [Run {
            first: 1,
            last: TOP - 2,
        }]
    );
}

#[test]
fn missing_of_a_set_with_gaps_lists_them_as_maximal_runs() {
    // Arrange
    let chunk_set = ChunkSet::of(&[2, 3, 7]);

    // Act
    let missing = chunk_set.missing(10);

    // Assert
    assert_eq!(
        missing,
        [
            Run { first: 1, last: 1 },
            Run { first: 4, last: 6 },
            Run { first: 8, last: 10 },
        ]
    );
}

#[test]
fn missing_of_the_empty_set_is_one_run_of_every_chunk() {
    // Arrange & Act
    let missing = ChunkSet::default().missing(TOP);

    // Assert
    assert_eq!(
        missing,
        [Run {
            first: 1,
            last: TOP
        }]
    );
}

#[test]
fn with_of_a_run_below_one_that_ends_at_the_top_of_u64_joins_them() {
    // Arrange
    let chunk_set = ChunkSet::default().with(Run {
        first: TOP,
        last: TOP,
    });

    // Act
    let joined = chunk_set.with(Run {
        first: TOP - 2,
        last: TOP - 1,
    });

    // Assert
    assert_eq!(
        joined.missing(TOP),
        [Run {
            first: 1,
            last: TOP - 3
        }]
    );
}

#[test]
fn with_of_runs_overlapping_and_touching_holds_each_chunk_once() {
    // Arrange
    let chunk_set = ChunkSet::default()
        .with(Run { first: 5, last: 7 })
        .with(Run { first: 1, last: 2 })
        .with(Run { first: 2, last: 4 })
        .with(Run { first: 6, last: 6 });

    // Act
    let held = (chunk_set.held(), chunk_set.ids());

    // Assert
    assert_eq!(held, (7, vec![1, 2, 3, 4, 5, 6, 7]));
}
