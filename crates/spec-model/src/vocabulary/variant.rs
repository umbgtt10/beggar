// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use serde_json::Map;
use serde_json::Value;

// A value of an enumeration (§1): an object with exactly one member, named for
// the value.
pub struct Variant;

impl Variant {
    #[must_use]
    pub fn of(name: &str, payload: Value) -> Value {
        Value::Object([(String::from(name), payload)].into_iter().collect())
    }

    #[must_use]
    pub fn unit(name: &str) -> Value {
        Self::of(name, Value::Object(Map::new()))
    }
}
