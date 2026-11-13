// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_spec_model::vocabulary::variant::Variant;
use serde_json::json;

#[test]
fn of_a_name_and_a_payload_is_an_object_of_one_member_named_for_it() {
    // Arrange & Act
    let variant = Variant::of("RecoveryAdvanced", json!({"held": "2", "chunks": "4"}));

    // Assert
    assert_eq!(
        variant,
        json!({"RecoveryAdvanced": {"held": "2", "chunks": "4"}})
    );
}

#[test]
fn unit_of_a_name_carries_the_empty_object() {
    // Arrange & Act
    let variant = Variant::unit("RecoveryCompleted");

    // Assert
    assert_eq!(variant, json!({"RecoveryCompleted": {}}));
}
