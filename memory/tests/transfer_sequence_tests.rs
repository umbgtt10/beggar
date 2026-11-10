// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_memory::counting_allocator::CountingAllocator;
use beggar_memory::transfer_sequence::TransferSequence;

const RECOVERIES: u64 = 100;

#[test]
fn run_of_transfers_in_order_holds_after_the_last_what_it_held_after_the_first() {
    // Arrange
    let sequence = TransferSequence::new(&CountingAllocator, (1..=8).collect());

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
    assert_eq!(profile.after_all, profile.after_first, "{profile:?}");
}

#[test]
fn run_of_transfers_interleaved_holds_after_the_last_what_it_held_after_the_first() {
    // Arrange
    let sequence = TransferSequence::new(&CountingAllocator, vec![1, 3, 5, 7, 2, 4, 6, 8]);

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
    assert_eq!(profile.after_all, profile.after_first, "{profile:?}");
}
