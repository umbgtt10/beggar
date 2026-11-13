// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde_json::from_str;
use serde_json::Value;
use std::fs::read_to_string;
use std::path::PathBuf;

// The specification's decision table, at the repository root.
pub struct SpecFiles;

impl SpecFiles {
    pub fn matrix() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/matrix.json");
        let text =
            read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    }
}
