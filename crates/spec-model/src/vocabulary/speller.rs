// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::vocabulary::spelling::Spelling;

// A spelling read either way: a value's name, or the value a name spells.
pub struct Speller;

impl Speller {
    #[must_use]
    pub fn name<W: Copy + PartialEq>(spelling: &Spelling<W>, word: W) -> &'static str {
        spelling
            .words
            .iter()
            .find(|(spelled, _)| *spelled == word)
            .map(|(_, name)| *name)
            .expect(spelling.kind)
    }

    #[must_use]
    pub fn of<W: Copy + PartialEq>(spelling: &Spelling<W>, name: &str) -> W {
        spelling
            .words
            .iter()
            .find(|(_, spelled)| *spelled == name)
            .map(|(word, _)| *word)
            .expect(spelling.kind)
    }
}
