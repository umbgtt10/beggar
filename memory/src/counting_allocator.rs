// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::allocation_counter::AllocationCounter;
use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static LIVE: Cell<isize> = const { Cell::new(0) };
    static PEAK: Cell<isize> = const { Cell::new(0) };
}

// The system allocator, counting the bytes each thread holds from when that
// thread starts counting: tests running in parallel in one binary never count
// each other's allocations, and a count that mixed them would be flaky, and
// flaky towards passing. A reallocation is the trait's own -- a new block, a
// copy, the old block freed -- so its moment holding both is counted.
pub struct CountingAllocator;

impl CountingAllocator {
    fn counted(change: isize) {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let live = LIVE.with(Cell::get) + change;
            LIVE.with(|cell| cell.set(live));
            PEAK.with(|cell| cell.set(cell.get().max(live)));
        }
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            Self::counted(layout.size() as isize);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        Self::counted(-(layout.size() as isize));
    }
}

impl AllocationCounter for CountingAllocator {
    fn start(&self) {
        LIVE.with(|cell| cell.set(0));
        PEAK.with(|cell| cell.set(0));
        COUNTING.with(|cell| cell.set(true));
    }

    fn stop(&self) {
        COUNTING.with(|cell| cell.set(false));
    }

    fn live(&self) -> isize {
        LIVE.with(Cell::get)
    }

    fn peak(&self) -> isize {
        PEAK.with(Cell::get)
    }
}
