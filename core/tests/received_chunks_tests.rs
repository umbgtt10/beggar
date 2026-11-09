// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::received_chunks::ReceivedChunks;

#[test]
fn ids_of_a_new_set_are_none() {
    // Arrange
    let received = ReceivedChunks::new();

    // Act
    let ids = received.ids();

    // Assert
    assert!(ids.is_empty());
}
