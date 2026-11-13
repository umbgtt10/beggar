// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_spec_model::value_reader::ValueReader;
use serde_json::json;
use serde_json::Value;

#[test]
fn list_of_a_list_is_its_items_in_order() {
    // Arrange
    let value = json!(["2", "1"]);

    // Act
    let items: Vec<&Value> = ValueReader::list(&value).collect();

    // Assert
    assert_eq!(items, [&json!("2"), &json!("1")]);
}

#[test]
#[should_panic(expected = "a list")]
fn list_of_a_value_that_is_not_a_list_panics() {
    // Arrange & Act & Assert
    let _ = ValueReader::list(&json!({"first": "1"})).count();
}

#[test]
#[should_panic(expected = "a u64 in decimal digits")]
fn number_of_a_string_past_the_top_of_u64_panics() {
    // Arrange & Act & Assert
    let _ = ValueReader::number(&json!("18446744073709551616"));
}

#[test]
fn number_of_the_top_of_u64_reads_it() {
    // Arrange & Act
    let number = ValueReader::number(&json!("18446744073709551615"));

    // Assert
    assert_eq!(number, u64::MAX);
}

#[test]
#[should_panic(expected = "a string")]
fn string_of_a_number_panics() {
    // Arrange & Act & Assert
    let _ = ValueReader::string(&json!(4));
}

#[test]
fn string_of_a_string_is_its_text() {
    // Arrange
    let value = json!("no-chunks");

    // Act
    let text = ValueReader::string(&value);

    // Assert
    assert_eq!(text, "no-chunks");
}
