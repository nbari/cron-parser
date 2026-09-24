#![allow(clippy::expect_used, unsafe_code)]

use chrono::{TimeZone, Utc};
use cron_parser::Schedule;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

#[test]
fn compiled_utc_query_does_not_allocate() {
    let schedule: Schedule = "*/5 * * * *".parse().expect("valid schedule");
    let cursor = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .expect("valid cursor");

    ALLOCATIONS.store(0, Ordering::Relaxed);
    let result = schedule.next_after(&cursor);
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);

    assert!(result.is_some());
    assert_eq!(allocations, 0);
}
