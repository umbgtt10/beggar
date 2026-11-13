// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// What the memory tier measures with: the bytes a thread holds, counted from
// when it starts counting, and the most it held at any instant since -- or
// since the peak was last counted again from what it held then.
pub trait AllocationCounter {
    fn start(&self);

    fn stop(&self);

    fn live(&self) -> isize;

    fn peak(&self) -> isize;

    fn rebase_peak(&self);
}
