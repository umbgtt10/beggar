// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// How the values of one of the vocabulary's enumerations are spelled (§1): each
// value and the name it is spelled with, and what a value of the table is, to
// be said when a reading finds none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spelling<W: 'static> {
    pub kind: &'static str,
    pub words: &'static [(W, &'static str)],
}
