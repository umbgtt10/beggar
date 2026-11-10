// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What a run of recoveries left the counter showing, in bytes counted from
// before the instance was created: what the instance held empty; the most it
// held between two calls -- the instance and one session at its worst; the
// most it held at any instant, inside a call included; and what it held after
// the first recovery and after the last.
#[derive(Debug)]
pub struct MemoryProfile {
    pub skeleton: isize,
    pub held_at_worst: isize,
    pub peak: isize,
    pub after_first: isize,
    pub after_all: isize,
}
