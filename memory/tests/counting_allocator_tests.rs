// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_memory::allocation_counter::AllocationCounter;
use beggar_memory::counting_allocator::CountingAllocator;
use std::alloc::GlobalAlloc;
use std::alloc::Layout;

fn layout(size: usize) -> Layout {
    Layout::from_size_align(size, 8).expect("a layout")
}

#[test]
fn alloc_after_stop_is_not_counted() {
    // Arrange
    let allocator = CountingAllocator;
    allocator.start();
    allocator.stop();

    // Act
    let pointer = unsafe { allocator.alloc(layout(64)) };

    // Assert
    assert_eq!(allocator.live(), 0);
    assert_eq!(allocator.peak(), 0);
    unsafe { allocator.dealloc(pointer, layout(64)) };
}

#[test]
fn alloc_while_counting_adds_its_size_to_live_and_to_peak() {
    // Arrange
    let allocator = CountingAllocator;
    allocator.start();

    // Act
    let pointer = unsafe { allocator.alloc(layout(64)) };

    // Assert
    assert_eq!(allocator.live(), 64);
    assert_eq!(allocator.peak(), 64);
    allocator.stop();
    unsafe { allocator.dealloc(pointer, layout(64)) };
}

#[test]
fn dealloc_while_counting_takes_its_size_from_live_and_leaves_the_peak() {
    // Arrange
    let allocator = CountingAllocator;
    allocator.start();
    let first = unsafe { allocator.alloc(layout(64)) };
    let second = unsafe { allocator.alloc(layout(32)) };

    // Act
    unsafe { allocator.dealloc(first, layout(64)) };

    // Assert
    assert_eq!(allocator.live(), 32);
    assert_eq!(allocator.peak(), 96);
    allocator.stop();
    unsafe { allocator.dealloc(second, layout(32)) };
}

#[test]
fn start_counts_from_zero_again() {
    // Arrange
    let allocator = CountingAllocator;
    allocator.start();
    let pointer = unsafe { allocator.alloc(layout(64)) };

    // Act
    allocator.start();

    // Assert
    assert_eq!(allocator.live(), 0);
    assert_eq!(allocator.peak(), 0);
    allocator.stop();
    unsafe { allocator.dealloc(pointer, layout(64)) };
}
