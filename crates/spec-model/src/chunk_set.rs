// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::run::Run;

// The chunk ids a session has accounted at this end (§7), kept as their
// maximal runs (§1) -- the one list of runs a set has -- so a run reaching the
// top of u64 is held as two numbers and never walked.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChunkSet {
    runs: Vec<Run>,
}

impl ChunkSet {
    #[must_use]
    pub fn of(ids: &[u64]) -> Self {
        ids.iter().fold(Self::default(), |set, id| {
            set.with(Run {
                first: *id,
                last: *id,
            })
        })
    }

    #[must_use]
    pub fn held(&self) -> u64 {
        self.runs.iter().map(|run| run.last - run.first + 1).sum()
    }

    // Every chunk of the run held: a run with no gap lies inside one maximal
    // run, or it reaches a chunk the set does not hold.
    #[must_use]
    pub fn holds(&self, run: Run) -> bool {
        self.runs
            .iter()
            .any(|held| held.first <= run.first && run.last <= held.last)
    }

    #[must_use]
    pub fn ids(&self) -> Vec<u64> {
        self.runs
            .iter()
            .flat_map(|run| run.first..=run.last)
            .collect()
    }

    // The ids of `1..=chunks` the set does not hold, as their maximal runs: the
    // gaps before, between and after the runs held.
    #[must_use]
    pub fn missing(&self, chunks: u64) -> Vec<Run> {
        let mut missing = Vec::new();
        let mut next = Some(1);
        for run in &self.runs {
            if let Some(first) = next.filter(|first| *first < run.first) {
                missing.push(Run {
                    first,
                    last: run.first - 1,
                });
            }
            next = run.last.checked_add(1);
        }
        if let Some(first) = next.filter(|first| *first <= chunks) {
            missing.push(Run {
                first,
                last: chunks,
            });
        }
        missing
    }

    // The set with the run's chunks added: the runs in order of their first
    // chunk, each that overlaps or touches the one before joined to it.
    #[must_use]
    pub fn with(&self, run: Run) -> Self {
        let mut runs: Vec<Run> = self.runs.iter().copied().chain([run]).collect();
        runs.sort_by_key(|run| run.first);
        let mut joined: Vec<Run> = Vec::new();
        for run in runs {
            match joined.last_mut() {
                Some(last) if run.first <= last.last.saturating_add(1) => {
                    last.last = last.last.max(run.last);
                }
                _ => joined.push(run),
            }
        }
        Self { runs: joined }
    }
}
