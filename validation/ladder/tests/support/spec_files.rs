// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde_json::from_str;
use serde_json::Value;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

// The specification, at the repository root beside the crate: read by the
// tests alone, never by the crate.
pub struct SpecFiles;

impl SpecFiles {
    pub fn matrix() -> Value {
        Self::read(&Self::root().join("matrix.json"))
    }

    pub fn schema() -> Value {
        Self::read(&Self::root().join("schema.json"))
    }

    // Every vector under its file's name, in the order of the names.
    pub fn vectors() -> Vec<(String, Value)> {
        let folder = Self::root().join("vectors");
        let mut files: Vec<PathBuf> = read_dir(&folder)
            .unwrap_or_else(|error| panic!("{}: {error}", folder.display()))
            .map(|entry| entry.expect("a vector's entry").path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect();
        files.sort();
        files
            .into_iter()
            .map(|path| {
                let name = path
                    .file_stem()
                    .expect("a vector's name")
                    .to_string_lossy()
                    .into_owned();
                (name, Self::read(&path))
            })
            .collect()
    }

    fn read(path: &Path) -> Value {
        let text =
            read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    }

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec")
    }
}
