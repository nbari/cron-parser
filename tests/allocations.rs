#![allow(clippy::expect_used, unsafe_code)]

use cron_parser::Schedule;
use jiff::{ZonedDateTime, civil, tz::TimeZone};
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

fn allocations_during(query: impl FnOnce() -> Option<ZonedDateTime>) -> usize {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    let result = query();
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    assert!(result.is_some());
    allocations
}

// A single test, so no other test thread allocates while counting.
#[test]
fn compiled_queries_do_not_allocate() {
    let schedule: Schedule = "*/5 * * * *".parse().expect("valid schedule");
    let utc = civil::datetime(2026, 1, 1, 0, 0, 0, 0)
        .to_zoned_datetime(TimeZone::UTC)
        .expect("valid cursor");
    let new_york = utc.with_time_zone(TimeZone::get("America/New_York").expect("known time zone"));

    assert_eq!(allocations_during(|| schedule.next_after(&utc)), 0);
    assert_eq!(allocations_during(|| schedule.previous_before(&utc)), 0);
    assert_eq!(allocations_during(|| schedule.next_after(&new_york)), 0);
    assert_eq!(
        allocations_during(|| schedule.previous_before(&new_york)),
        0
    );
}
