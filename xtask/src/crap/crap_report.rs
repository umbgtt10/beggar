// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::crap::crap_function::CrapFunction;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CrapReport {
    total_functions: u32,
    crappy_functions: u32,
    crappy_percent: f64,
    functions: Vec<CrapFunction>,
}

impl CrapReport {
    pub fn crappy_functions(&self) -> u32 {
        self.crappy_functions
    }

    pub fn crappy_percent(&self) -> f64 {
        self.crappy_percent
    }

    pub fn functions(&self) -> &[CrapFunction] {
        &self.functions
    }

    pub fn is_clean(&self) -> bool {
        self.crappy_functions == 0
    }

    pub fn offenders(&self) -> Vec<&CrapFunction> {
        self.functions
            .iter()
            .filter(|function| !function.is_clean())
            .collect()
    }

    pub fn summary(&self) -> String {
        format!(
            "crap4rust: {}/{} functions crappy ({:.1}%)",
            self.crappy_functions, self.total_functions, self.crappy_percent
        )
    }

    pub fn total_functions(&self) -> u32 {
        self.total_functions
    }
}
