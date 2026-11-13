// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::chunk_set::ChunkSet;

const TOP: u64 = u64::MAX;

fn holding(runs: &[(u64, u64)]) -> ChunkSet {
    let mut set = ChunkSet::new();
    for (first, last) in runs {
        set.add(*first, *last);
    }
    set
}

#[test]
fn add_at_the_top_of_u64_joins_runs_without_overflowing() {
    // Arrange
    let mut set = holding(&[(TOP, TOP)]);

    // Act
    set.add(TOP - 1, TOP - 1);

    // Assert
    assert_eq!(set.ids(), vec![TOP - 1, TOP]);
    assert_eq!(set.count(), 2);
}

#[test]
fn add_of_a_run_already_held_changes_nothing() {
    // Arrange
    let mut set = holding(&[(2, 6)]);

    // Act
    set.add(3, 4);

    // Assert
    assert_eq!(set, holding(&[(2, 6)]));
    assert_eq!(set.count(), 5);
}

#[test]
fn add_of_a_run_apart_from_the_others_keeps_the_gaps() {
    // Arrange
    let mut set = holding(&[(1, 2), (8, 9)]);

    // Act
    set.add(5, 5);

    // Assert
    assert_eq!(set.ids(), vec![1, 2, 5, 8, 9]);
    assert_eq!(set.count(), 5);
}

#[test]
fn add_of_a_run_bridging_two_joins_all_three() {
    // Arrange
    let mut set = holding(&[(1, 2), (5, 6)]);

    // Act
    set.add(3, 4);

    // Assert
    assert_eq!(set, holding(&[(1, 6)]));
    assert_eq!(set.count(), 6);
}

#[test]
fn add_of_a_run_overlapping_another_counts_each_chunk_once() {
    // Arrange
    let mut set = holding(&[(2, 4)]);

    // Act
    set.add(3, 6);

    // Assert
    assert_eq!(set.ids(), vec![2, 3, 4, 5, 6]);
    assert_eq!(set.count(), 5);
}

#[test]
fn add_of_a_run_to_an_empty_set_holds_its_chunks() {
    // Arrange
    let mut set = ChunkSet::new();

    // Act
    set.add(3, 5);

    // Assert
    assert_eq!(set.ids(), vec![3, 4, 5]);
    assert_eq!(set.count(), 3);
}

#[test]
fn add_of_a_run_touching_another_joins_it_on_either_side() {
    // Arrange
    let mut set = holding(&[(4, 5)]);

    // Act
    set.add(6, 7);
    set.add(2, 3);

    // Assert
    assert_eq!(set, holding(&[(2, 7)]));
    assert_eq!(set.ids(), vec![2, 3, 4, 5, 6, 7]);
}

#[test]
fn add_of_runs_in_any_order_holds_one_and_the_same_set() {
    // Arrange
    let mut set = holding(&[(7, 8), (1, 2), (4, 4)]);

    // Act
    set.add(3, 3);

    // Assert
    assert_eq!(set, holding(&[(1, 4), (7, 8)]));
    assert_eq!(set.ids(), vec![1, 2, 3, 4, 7, 8]);
}

#[test]
fn count_of_a_new_set_is_zero() {
    // Arrange
    let set = ChunkSet::new();

    // Act
    let count = set.count();

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn count_of_every_chunk_up_to_the_top_of_u64_is_u64_max() {
    // Arrange
    let set = holding(&[(1, TOP)]);

    // Act
    let count = set.count();

    // Assert
    assert_eq!(count, TOP);
}

#[test]
fn ids_of_a_new_set_are_none() {
    // Arrange
    let set = ChunkSet::new();

    // Act
    let ids = set.ids();

    // Assert
    assert!(ids.is_empty());
}

#[test]
fn missing_of_a_full_set_is_none() {
    // Arrange
    let set = holding(&[(1, 4)]);

    // Act
    let missing = set.missing(4);

    // Assert
    assert!(missing.is_empty());
}

#[test]
fn missing_of_a_new_set_is_one_run_of_every_chunk() {
    // Arrange
    let set = ChunkSet::new();

    // Act
    let missing = set.missing(1_000_000);

    // Assert
    assert_eq!(missing, vec![1..=1_000_000]);
}

#[test]
fn missing_of_a_set_holding_its_first_and_last_chunks_is_the_gap_between() {
    // Arrange
    let set = holding(&[(1, 2), (9, 10)]);

    // Act
    let missing = set.missing(10);

    // Assert
    assert_eq!(missing, vec![3..=8]);
}

#[test]
fn missing_of_a_set_holding_the_top_of_u64_ends_below_it() {
    // Arrange
    let set = holding(&[(3, TOP)]);

    // Act
    let missing = set.missing(TOP);

    // Assert
    assert_eq!(missing, vec![1..=2]);
}

#[test]
fn missing_of_a_set_with_gaps_is_its_gaps_as_maximal_runs() {
    // Arrange
    let set = holding(&[(3, 4), (7, 7)]);

    // Act
    let missing = set.missing(10);

    // Assert
    assert_eq!(missing, vec![1..=2, 5..=6, 8..=10]);
}

#[test]
fn missing_up_to_the_top_of_u64_reaches_it() {
    // Arrange
    let set = holding(&[(TOP - 1, TOP - 1)]);

    // Act
    let missing = set.missing(TOP);

    // Assert
    assert_eq!(missing, vec![1..=TOP - 2, TOP..=TOP]);
}
