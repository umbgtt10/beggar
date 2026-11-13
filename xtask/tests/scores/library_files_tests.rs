// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::scores::library_files::LibraryFiles;

const DIRECTORY: &str = "crates";
const LIBRARY: [&str; 2] = ["core", "spec-model"];

fn listing(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|path| String::from(*path)).collect()
}

#[test]
fn crates_of_a_listing_leaves_out_a_crate_with_no_source_file() {
    // Arrange
    let files = listing(&[
        "crates/core/src/lib.rs",
        "crates/spec-model/Cargo.toml",
        "crates/spec-model/tests/all_tests.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(DIRECTORY, &LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["core"]);
}

#[test]
fn crates_of_a_listing_names_each_crate_with_a_source_file_once_in_the_library_order() {
    // Arrange
    let files = listing(&[
        "crates/spec-model/src/model.rs",
        "crates/core/Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/core/src/registry.rs",
        "crates/spec-model/tests/model_tests.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(DIRECTORY, &LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["core", "spec-model"]);
}

#[test]
fn crates_of_a_listing_reads_only_the_library_crates() {
    // Arrange
    let files = listing(&[
        "validation/core-validation/src/lib.rs",
        "validation/ladder/src/lib.rs",
        "validation/memory/src/lib.rs",
        "core/src/lib.rs",
        "crates/spec-model/src/lib.rs",
    ]);

    // Act
    let crates = LibraryFiles::crates(DIRECTORY, &LIBRARY, &files);

    // Assert
    assert_eq!(crates, ["spec-model"]);
}

#[test]
fn crates_of_a_listing_with_no_source_is_empty() {
    // Arrange
    let files = listing(&["crates/core/Cargo.toml", "crates/spec-model/README.md"]);

    // Act
    let crates = LibraryFiles::crates(DIRECTORY, &LIBRARY, &files);

    // Assert
    assert!(crates.is_empty(), "{crates:?}");
}
