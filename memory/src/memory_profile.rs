// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What a run of recoveries left the counter showing, in bytes counted from
// before the instance was created: what the instance held empty; the most it
// held between two calls -- the instance and one session at its worst; the
// most it held at any instant, inside a call included, and the most from the
// moment it was full; what it held once full -- keeping as many finished
// sessions as its retention, after the first recovery at retention 0 and
// after as many recoveries as the retention otherwise; and what it held after
// the last.
#[derive(Debug)]
pub struct MemoryProfile {
    pub skeleton: isize,
    pub held_at_worst: isize,
    pub peak: isize,
    pub peak_once_full: isize,
    pub after_full: isize,
    pub after_all: isize,
}
