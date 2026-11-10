// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::rejection::Rejection;
use beggar::start_rows::StartRows;

const NODE: u64 = 1;

#[test]
fn refusal_of_a_start_at_the_bound_is_registry_full_carrying_it() {
    // Arrange
    let rows = StartRows::new(NODE, 2, Some(2));

    // Act
    let refusal = rows.refusal(4, NODE, 2);

    // Assert
    assert_eq!(refusal, Some(Rejection::RegistryFull { capacity: 2 }));
}

#[test]
fn refusal_of_a_start_between_two_other_nodes_is_not_a_party() {
    // Arrange
    let rows = StartRows::new(NODE, 0, None);

    // Act
    let refusal = rows.refusal(4, 2, 3);

    // Assert
    assert_eq!(refusal, Some(Rejection::NotAParty));
}

#[test]
fn refusal_of_a_start_under_the_bound_is_none() {
    // Arrange
    let rows = StartRows::new(NODE, 1, Some(2));

    // Act
    let refusal = rows.refusal(4, 2, NODE);

    // Assert
    assert_eq!(refusal, None);
}

#[test]
fn refusal_of_a_start_with_no_chunks_is_no_chunks_before_any_other_row() {
    // Arrange
    let rows = StartRows::new(NODE, 0, Some(0));

    // Act
    let refusal = rows.refusal(0, 2, 2);

    // Assert
    assert_eq!(refusal, Some(Rejection::NoChunks));
}

#[test]
fn refusal_of_a_start_with_one_end_twice_is_self_addressed_before_not_a_party() {
    // Arrange
    let rows = StartRows::new(NODE, 0, None);

    // Act
    let refusal = rows.refusal(4, 2, 2);

    // Assert
    assert_eq!(refusal, Some(Rejection::SelfAddressed));
}
