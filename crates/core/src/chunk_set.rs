// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;
use core::ops::RangeInclusive;

// A session's chunk set -- the chunk ids accounted at this end -- shown through
// behaviour and never its fields. It keeps maximal runs, sorted, so its size
// follows its gaps and not its chunks, and two sets holding the same ids are
// equal however they were built.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChunkSet {
    runs: Vec<RangeInclusive<u64>>,
}

impl ChunkSet {
    pub fn new() -> Self {
        Self { runs: Vec::new() }
    }

    // Adds the run first..=last, last not below first: the runs it touches or
    // overlaps, found by two binary searches, are replaced in place by one.
    pub fn add(&mut self, first: u64, last: u64) {
        let start = self
            .runs
            .partition_point(|run| run.end().saturating_add(1) < first);
        let end = self
            .runs
            .partition_point(|run| *run.start() <= last.saturating_add(1));
        let merged = self.runs[start..end]
            .iter()
            .fold(first..=last, |merged, run| {
                *merged.start().min(run.start())..=*merged.end().max(run.end())
            });
        self.runs.splice(start..end, [merged]);
    }

    pub fn count(&self) -> u64 {
        self.runs
            .iter()
            .map(|run| run.end() - run.start() + 1)
            .sum()
    }

    // The chunk ids held, in ascending order.
    pub fn ids(&self) -> Vec<u64> {
        self.runs.iter().flat_map(|run| run.clone()).collect()
    }

    // The chunk ids of 1..=chunks not held, as their maximal runs.
    // Each gap before a run, then what follows the last run up to chunks; a
    // run ending at the top of u64 leaves nothing after it.
    pub fn missing(&self, chunks: u64) -> Vec<RangeInclusive<u64>> {
        let mut missing = Vec::new();
        let mut next = Some(1);
        for run in &self.runs {
            if let Some(from) = next.filter(|from| from < run.start()) {
                missing.push(from..=run.start() - 1);
            }
            next = run.end().checked_add(1);
        }
        if let Some(from) = next.filter(|from| *from <= chunks) {
            missing.push(from..=chunks);
        }
        missing
    }
}
