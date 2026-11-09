// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The conformance tier: every vector of the specification, run against the
// crate as the specification states it -- each step's receipt, then the final
// state a closing ShowRegistry answers -- every vector and the matrix valid
// against the schema, and every row of the matrix exercised.

use crate::support::spec_files::SpecFiles;
use crate::support::spec_reader::SpecReader;
use crate::support::spec_schema::SpecSchema;
use crate::support::spec_writer::SpecWriter;
use beggar::beggar::Beggar;
use beggar::command::Command;
use serde_json::json;
use serde_json::Value;
use std::collections::BTreeSet;

// What a vector's run answered where the vector expected otherwise: nothing
// for a conforming implementation.
fn departures(name: &str, vector: &Value) -> Vec<String> {
    let mut beggar: Beggar<()> = Beggar::new(SpecReader::origin(&vector["origin"]));
    let mut departures = Vec::new();
    let steps = vector["steps"].as_array().expect("a vector's steps");
    for (index, step) in steps.iter().enumerate() {
        let receipt = SpecWriter::receipt(&beggar.process(SpecReader::command(&step["command"])));
        if receipt != step["receipt"] {
            departures.push(format!(
                "{name}, step {}: answered {receipt}, not {}",
                index + 1,
                step["receipt"]
            ));
        }
    }
    let closing = SpecWriter::receipt(&beggar.process(Command::ShowRegistry));
    let expected = json!({ "RegistryShown": vector["final"] });
    if closing["output"] != expected {
        departures.push(format!(
            "{name}, final state: answered {}, not {expected}",
            closing["output"]
        ));
    }
    departures
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
fn process_of_every_vector_answers_its_receipts_and_its_final_state() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .flat_map(|(name, vector)| departures(name, vector))
        .collect();

    // Assert
    assert!(!vectors.is_empty(), "no vector was read");
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn vectors_of_the_specification_exercise_every_row_of_the_matrix() {
    // Arrange
    let rows: BTreeSet<String> = SpecFiles::matrix()["rows"]
        .as_array()
        .expect("the matrix's rows")
        .iter()
        .map(|row| String::from(row["id"].as_str().expect("a row's id")))
        .collect();

    // Act
    let exercised: BTreeSet<String> = SpecFiles::vectors()
        .iter()
        .flat_map(|(_, vector)| {
            vector["rows"]
                .as_array()
                .expect("a vector's rows")
                .iter()
                .map(|row| String::from(row.as_str().expect("a row's id")))
                .collect::<Vec<String>>()
        })
        .collect();

    // Assert
    let unexercised: Vec<&String> = rows.difference(&exercised).collect();
    let unknown: Vec<&String> = exercised.difference(&rows).collect();
    assert!(
        unexercised.is_empty(),
        "rows no vector exercises: {unexercised:?}"
    );
    assert!(
        unknown.is_empty(),
        "rows the matrix does not have: {unknown:?}"
    );
}
