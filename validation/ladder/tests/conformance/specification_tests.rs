// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The specification as files: the matrix and every vector valid against the
// schema, the released vectors valid against it too, and every released
// vector still there, byte for byte -- a released vector is never removed or
// edited.

use crate::support::spec_files::SpecFiles;
use crate::support::spec_schema::SpecSchema;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Write;

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut written, byte| {
            write!(written, "{byte:02x}").expect("writing to a string");
            written
        })
}

#[test]
fn admits_matrix_of_the_specification_holds() {
    // Arrange
    let schema = SpecSchema::new(SpecFiles::schema());

    // Act
    let admitted = schema.admits_matrix(&SpecFiles::matrix());

    // Assert
    assert!(
        admitted,
        "matrix.json is not valid against the schema's matrix"
    );
}

#[test]
fn admits_released_of_the_specification_holds() {
    // Arrange
    let schema = SpecSchema::new(SpecFiles::schema());

    // Act & Assert
    assert!(schema.admits_released(&SpecFiles::released()));
}

#[test]
fn admits_vector_of_every_vector_holds() {
    // Arrange
    let schema = SpecSchema::new(SpecFiles::schema());

    // Act
    let refused: Vec<String> = SpecFiles::vectors()
        .into_iter()
        .filter(|(_, vector)| !schema.admits_vector(vector))
        .map(|(name, _)| name)
        .collect();

    // Assert
    assert!(
        refused.is_empty(),
        "vectors not valid against the schema: {refused:?}"
    );
}

#[test]
fn vector_bytes_of_every_released_vector_are_as_released() {
    // Arrange
    let released = SpecFiles::released();

    // Act
    let departures: Vec<String> = released["vectors"]
        .as_array()
        .expect("the released vectors")
        .iter()
        .filter_map(|entry| {
            let id = entry["id"].as_str().expect("a released vector's id");
            let expected = entry["sha256"].as_str().expect("its digest");
            match SpecFiles::vector_bytes(id) {
                None => Some(format!("{id}: released, and missing")),
                Some(bytes) if digest(&bytes) != expected => {
                    Some(format!("{id}: released, and changed"))
                }
                Some(_) => None,
            }
        })
        .collect();

    // Assert
    assert!(departures.is_empty(), "{departures:#?}");
}
