// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The conformance tier, the specification's model: every vector run against the
// independent model -- each step's receipt, then the final state a closing
// ShowRegistry answers, and every step but a query's answered alike with the
// retention raised to the top of u64 -- each vector deciding by exactly the
// rows it names and the vectors by every row of the matrix, and the model
// knowing exactly the matrix's rows. The vectors are checked here before the
// crate runs them.

use crate::support::spec_files::SpecFiles;
use beggar_spec_model::answer::Answer;
use beggar_spec_model::model::Model;
use beggar_spec_model::rules::rule_book::RuleBook;
use beggar_spec_model::spec_reader::SpecReader;
use serde_json::json;
use serde_json::Value;
use std::collections::BTreeSet;

const QUERIES: [&str; 4] = [
    "ListRecoveries",
    "ShowRecovery",
    "ListMissingChunks",
    "ShowRegistry",
];

// The model's answers to a vector's steps, from the origin given, and to the
// commands given after them.
fn answers(vector: &Value, origin: &Value, after: &[Value]) -> Vec<Answer> {
    let mut model = Model::new(
        SpecReader::table(&SpecFiles::matrix()),
        SpecReader::origin(origin),
    );
    steps(vector)
        .iter()
        .map(|step| &step["command"])
        .chain(after)
        .map(|command| model.process(command))
        .collect()
}

fn ids(rows: &Value) -> BTreeSet<String> {
    rows.as_array()
        .expect("a list of rows")
        .iter()
        .map(|row| String::from(row.as_str().expect("a row's id")))
        .collect()
}

fn matrix_row_ids() -> BTreeSet<String> {
    SpecFiles::matrix()["rows"]
        .as_array()
        .expect("the matrix's rows")
        .iter()
        .map(|row| String::from(row["id"].as_str().expect("a row's id")))
        .collect()
}

// A vector's origin, its retention raised to the top of u64.
fn raised(origin: &Value) -> Value {
    let mut raised = origin.clone();
    for kind in ["Fresh", "Checkpoint"] {
        if raised.get(kind).is_some() {
            raised[kind]["retention"] = json!("18446744073709551615");
        }
    }
    raised
}

fn steps(vector: &Value) -> &Vec<Value> {
    vector["steps"].as_array().expect("a vector's steps")
}

#[test]
fn process_of_every_vector_answers_each_step_as_written() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .flat_map(|(name, vector)| {
            answers(vector, &vector["origin"], &[])
                .into_iter()
                .zip(steps(vector))
                .enumerate()
                .filter(|(_, (answer, step))| answer.receipt != step["receipt"])
                .map(|(index, (answer, step))| {
                    format!(
                        "{name}, step {}: the model answers {}, the vector {}",
                        index + 1,
                        answer.receipt,
                        step["receipt"]
                    )
                })
                .collect::<Vec<String>>()
        })
        .collect();

    // Assert
    assert_eq!(vectors.len(), 79);
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn process_of_every_vector_answers_the_closing_show_registry_with_the_final_state() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .filter_map(|(name, vector)| {
            let answered = answers(vector, &vector["origin"], &[json!({"ShowRegistry": {}})]);
            let closing = &answered.last().expect("the closing answer").receipt["output"];
            let expected = json!({"RegistryShown": vector["final"]});
            (*closing != expected)
                .then(|| format!("{name}: the model answers {closing}, the vector {expected}"))
        })
        .collect();

    // Assert
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn process_of_every_vector_at_the_highest_retention_answers_every_step_but_a_query_s_alike() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .flat_map(|(name, vector)| {
            answers(vector, &raised(&vector["origin"]), &[])
                .into_iter()
                .zip(steps(vector))
                .enumerate()
                .filter(|(_, (answer, step))| {
                    let command = step["command"].as_object().expect("a command");
                    let query = QUERIES.iter().any(|query| command.contains_key(*query));
                    !query && answer.receipt != step["receipt"]
                })
                .map(|(index, (answer, step))| {
                    format!(
                        "{name}, step {} at the highest retention: the model answers {}, the vector {}",
                        index + 1,
                        answer.receipt,
                        step["receipt"]
                    )
                })
                .collect::<Vec<String>>()
        })
        .collect();

    // Assert
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn process_of_every_vector_decides_by_exactly_the_rows_it_names() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let departures: Vec<String> = vectors
        .iter()
        .filter_map(|(name, vector)| {
            let decided: BTreeSet<String> = answers(vector, &vector["origin"], &[])
                .into_iter()
                .map(|answer| answer.row)
                .collect();
            let named = ids(&vector["rows"]);
            (decided != named).then(|| format!("{name}: names {named:?}, decides by {decided:?}"))
        })
        .collect();

    // Assert
    assert!(departures.is_empty(), "{departures:#?}");
}

#[test]
fn process_of_the_vectors_decides_by_every_row_of_the_matrix() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let decided: BTreeSet<String> = vectors
        .iter()
        .flat_map(|(_, vector)| answers(vector, &vector["origin"], &[]))
        .map(|answer| answer.row)
        .collect();

    // Assert
    assert_eq!(decided, matrix_row_ids());
}

#[test]
fn row_ids_of_the_model_are_the_rows_of_the_matrix() {
    // Arrange & Act
    let known: BTreeSet<String> = RuleBook::row_ids().into_iter().map(String::from).collect();

    // Assert
    assert_eq!(known, matrix_row_ids());
}
