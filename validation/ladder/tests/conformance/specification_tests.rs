// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The specification as files: the matrix and every vector valid against the
// schema, the released vectors valid against it too, and every released
// vector still there, byte for byte -- a released vector is never removed or
// edited; and no word the vocabulary leaves out among the schema's names or
// the crate's public ones, as cartel's are checked.

use crate::support::spec_files::SpecFiles;
use crate::support::spec_schema::SpecSchema;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Write;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

// The words beggar's vocabulary leaves out (DESIGN.md §4): cartel's -- each one
// protocol's, a role, a phase's name, what is decided, a rule or evidence --
// and what recovery moves without beggar seeing it: a snapshot, a payload, its
// bytes, and the quorum a protocol counts.
const LEFT_OUT: [&str; 30] = [
    "term",
    "height",
    "slot",
    "view",
    "epoch",
    "leader",
    "candidate",
    "proposer",
    "validator",
    "prepare",
    "commit",
    "prevote",
    "block",
    "log",
    "entry",
    "index",
    "majority",
    "supermajority",
    "certificate",
    "signature",
    "proof",
    "elected",
    "prepared",
    "committed",
    "decided",
    "value",
    "snapshot",
    "payload",
    "bytes",
    "quorum",
];
const PUBLIC_ITEMS: [&str; 7] = [
    "pub struct ",
    "pub enum ",
    "pub fn ",
    "pub mod ",
    "pub trait ",
    "pub type ",
    "pub const ",
];

// The names the crate declares in public, from its source: its public items,
// the fields of its records and the values of its enumerations, each split
// into its words.
fn declared(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut in_enumeration = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        if in_enumeration && line == "}" {
            in_enumeration = false;
        } else if in_enumeration && line.starts_with("    ") && !line.starts_with("        ") {
            found.extend(words(identifier(trimmed)));
        }
        for item in PUBLIC_ITEMS {
            if let Some(rest) = trimmed.strip_prefix(item) {
                found.extend(words(identifier(rest)));
                in_enumeration |= item == "pub enum ";
            }
        }
        if let Some((field, _)) = trimmed
            .strip_prefix("pub ")
            .and_then(|rest| rest.split_once(':'))
            && field
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_')
        {
            found.extend(words(field));
        }
    }
    found
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut written, byte| {
            write!(written, "{byte:02x}").expect("writing to a string");
            written
        })
}

fn identifier(text: &str) -> &str {
    let end = text
        .find(|character: char| !(character.is_alphanumeric() || character == '_'))
        .unwrap_or(text.len());
    &text[..end]
}

// Every name the schema gives a definition, a member or a value of an
// enumeration, split into its words: `next_receipt` and `QuorumReached` into
// two each.
fn names(value: &Value, found: &mut Vec<String>) {
    if let Some(object) = value.as_object() {
        for (key, inner) in object {
            if !key.starts_with('$') {
                found.extend(words(key));
            }
            if key == "required" {
                for name in inner
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    found.extend(words(name));
                }
            }
            names(inner, found);
        }
    } else if let Some(items) = value.as_array() {
        for item in items {
            names(item, found);
        }
    }
}

// Every Rust source file under a folder, its subfolders included.
fn sources(folder: &Path) -> Vec<PathBuf> {
    read_dir(folder)
        .expect("a source folder")
        .map(|entry| entry.expect("an entry").path())
        .flat_map(|path| {
            if path.is_dir() {
                sources(&path)
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                vec![path]
            } else {
                Vec::new()
            }
        })
        .collect()
}

// A name's words, lowercase: split at each underscore and where a capital
// follows a lowercase letter or a digit, so `next_receipt`, `QuorumReached`
// and `BLOCK_HEIGHT` each give their two.
fn words(name: &str) -> Vec<String> {
    let mut split = Vec::new();
    let mut current = String::new();
    let mut after_lowercase = false;
    for character in name.chars() {
        let boundary = character == '_' || (character.is_uppercase() && after_lowercase);
        if boundary && !current.is_empty() {
            split.push(current.clone());
            current.clear();
        }
        after_lowercase = character.is_lowercase() || character.is_ascii_digit();
        if character != '_' {
            current.extend(character.to_lowercase());
        }
    }
    if !current.is_empty() {
        split.push(current);
    }
    split
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
fn declared_of_the_crate_holds_no_word_the_vocabulary_leaves_out() {
    // Arrange
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/core/src");

    // Act
    let found: Vec<String> = sources(&source)
        .into_iter()
        .map(|path| read_to_string(path).expect("a source file's text"))
        .flat_map(|text| declared(&text))
        .collect();

    // Assert
    let left_out: Vec<&String> = found
        .iter()
        .filter(|word| LEFT_OUT.contains(&word.as_str()))
        .collect();
    assert!(found.contains(&String::from("session")));
    assert!(left_out.is_empty(), "{left_out:#?}");
}

#[test]
fn names_of_the_schema_hold_no_word_the_vocabulary_leaves_out() {
    // Arrange
    let mut found = Vec::new();

    // Act
    names(&SpecFiles::schema(), &mut found);

    // Assert
    let left_out: Vec<&String> = found
        .iter()
        .filter(|word| LEFT_OUT.contains(&word.as_str()))
        .collect();
    assert!(!found.is_empty());
    assert!(left_out.is_empty(), "{left_out:#?}");
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
