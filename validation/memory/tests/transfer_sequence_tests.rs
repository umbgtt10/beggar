// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_memory::counting_allocator::CountingAllocator;
use beggar_memory::transfer_sequence::TransferSequence;

const RECOVERIES: u64 = 100;

#[test]
fn run_of_transfers_at_a_retention_holds_after_the_last_what_it_held_once_it_kept_as_many() {
    // Arrange
    let keeping_none =
        TransferSequence::new(&CountingAllocator, vec![1, 3, 5, 7, 2, 4, 6, 8], None, 0);
    let keeping = TransferSequence::new(&CountingAllocator, vec![1, 3, 5, 7, 2, 4, 6, 8], None, 5);

    // Act
    let profiles = [keeping_none.run(RECOVERIES), keeping.run(RECOVERIES)];

    // Assert
    let [none, kept] = profiles;
    assert!(
        kept.after_all > none.after_all,
        "keeping five held no more than keeping none: {kept:?}, {none:?}"
    );
    assert!(
        kept.peak_once_full <= kept.held_at_worst,
        "bytes held inside a call, once full, beyond the state it left: {kept:?}"
    );
    assert_eq!(kept.after_all, kept.after_full, "{kept:?}");
}

#[test]
fn run_of_transfers_cancelled_midway_at_a_retention_holds_after_the_last_what_it_held_once_it_kept_as_many(
) {
    // Arrange
    let sequence =
        TransferSequence::new(&CountingAllocator, vec![1, 3, 5, 7, 2, 4, 6, 8], Some(5), 5);

    // Act
    let profile = sequence.run(RECOVERIES);

    // Assert
    assert!(
        profile.after_full > profile.skeleton,
        "nothing was kept: {profile:?}"
    );
    assert!(
        profile.peak_once_full <= profile.held_at_worst,
        "bytes held inside a call, once full, beyond the state it left: {profile:?}"
    );
    assert_eq!(profile.after_all, profile.after_full, "{profile:?}");
}

#[test]
fn run_of_transfers_cancelled_midway_holds_after_the_last_what_it_held_after_the_first() {
    // Arrange
    let sequence =
        TransferSequence::new(&CountingAllocator, vec![1, 3, 5, 7, 2, 4, 6, 8], Some(5), 0);

    // Act
    let profile = sequence.run(RECOVERIES);

    // Assert
    assert_eq!(
        profile.skeleton, 0,
        "five idle instances hold bytes: {profile:?}"
    );
    assert!(
        profile.held_at_worst > profile.skeleton,
        "the counter never moved: {profile:?}"
    );
    assert!(
        profile.peak <= profile.held_at_worst,
        "bytes held inside a call beyond the state it left: {profile:?}"
    );
    assert_eq!(profile.after_all, profile.after_full, "{profile:?}");
}

#[test]
fn run_of_transfers_in_order_holds_after_the_last_what_it_held_after_the_first() {
    // Arrange
    let sequence = TransferSequence::new(&CountingAllocator, (1..=8).collect(), None, 0);

    // Act
    let profile = sequence.run(RECOVERIES);

    // Assert
    assert_eq!(
        profile.skeleton, 0,
        "five idle instances hold bytes: {profile:?}"
    );
    assert!(
        profile.held_at_worst > profile.skeleton,
        "the counter never moved: {profile:?}"
    );
    assert!(
        profile.peak <= profile.held_at_worst,
        "bytes held inside a call beyond the state it left: {profile:?}"
    );
    assert_eq!(profile.after_all, profile.after_full, "{profile:?}");
}

#[test]
fn run_of_transfers_interleaved_holds_after_the_last_what_it_held_after_the_first() {
    // Arrange
    let sequence = TransferSequence::new(&CountingAllocator, vec![1, 3, 5, 7, 2, 4, 6, 8], None, 0);

    // Act
    let profile = sequence.run(RECOVERIES);

    // Assert
    assert_eq!(
        profile.skeleton, 0,
        "five idle instances hold bytes: {profile:?}"
    );
    assert!(
        profile.held_at_worst > profile.skeleton,
        "the counter never moved: {profile:?}"
    );
    assert!(
        profile.peak <= profile.held_at_worst,
        "bytes held inside a call beyond the state it left: {profile:?}"
    );
    assert_eq!(profile.after_all, profile.after_full, "{profile:?}");
}
