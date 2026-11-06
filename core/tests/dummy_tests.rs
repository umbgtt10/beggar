// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::dummy::Dummy;

#[test]
fn name_of_the_dummy_returns_the_crate_name() {
    // Arrange
    let dummy = Dummy;

    // Act
    let name = dummy.name();

    // Assert
    assert_eq!(name, "beggar");
}
