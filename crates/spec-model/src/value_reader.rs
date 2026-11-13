// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde_json::Value;

// The specification's encoding read at its smallest: a string, a u64 written
// as one in decimal digits (§1), and a list. A value of another shape is
// outside the specification, and reading it panics.
pub struct ValueReader;

impl ValueReader {
    pub fn list(value: &Value) -> impl Iterator<Item = &Value> {
        value.as_array().expect("a list").iter()
    }

    #[must_use]
    pub fn number(value: &Value) -> u64 {
        Self::string(value)
            .parse()
            .expect("a u64 in decimal digits")
    }

    #[must_use]
    pub fn string(value: &Value) -> &str {
        value.as_str().expect("a string")
    }
}
