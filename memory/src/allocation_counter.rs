// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What the memory tier measures with: the bytes a thread holds, counted from
// when it starts counting, and the most it held at any instant since.
pub trait AllocationCounter {
    fn start(&self);

    fn stop(&self);

    fn live(&self) -> isize;

    fn peak(&self) -> isize;
}
