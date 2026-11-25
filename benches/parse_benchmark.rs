#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use criterion::{Criterion, criterion_group, criterion_main};
use cron_parser::{Schedule, parse};
use jiff::{civil, tz::TimeZone};
use std::{hint::black_box, str::FromStr};

pub fn criterion_benchmark(criterion: &mut Criterion) {
    let cursor = civil::datetime(2026, 1, 1, 0, 0, 0, 0)
        .to_zoned_datetime(TimeZone::UTC)
        .expect("benchmark date");

    criterion.bench_function("compile/dense", |bencher| {
        bencher.iter(|| Schedule::from_str(black_box("*/5 * * * *")));
    });

    criterion.bench_function("one-shot/dense", |bencher| {
        bencher.iter(|| parse(black_box("*/5 * * * *"), black_box(&cursor)));
    });

    let dense = Schedule::from_str("*/5 * * * *").expect("valid schedule");
    criterion.bench_function("compiled-next/dense", |bencher| {
        bencher.iter(|| dense.next_after(black_box(&cursor)));
    });

    criterion.bench_function("compiled-previous/dense", |bencher| {
        bencher.iter(|| dense.previous_before(black_box(&cursor)));
    });

    criterion.bench_function("iterate/dense-100", |bencher| {
        bencher.iter(|| dense.after(black_box(&cursor)).take(100).count());
    });

    // The last slot of the day and month has passed, so the search has to
    // move on to the next month.
    let late_cursor = civil::datetime(2026, 1, 31, 23, 57, 0, 0)
        .to_zoned_datetime(TimeZone::UTC)
        .expect("benchmark date");
    criterion.bench_function("compiled-next/late-in-day", |bencher| {
        bencher.iter(|| dense.next_after(black_box(&late_cursor)));
    });

    let new_york = TimeZone::get("America/New_York").expect("benchmark time zone");
    let zoned_cursor = civil::datetime(2026, 6, 15, 12, 2, 0, 0)
        .to_zoned_datetime(new_york.clone())
        .expect("benchmark date");
    criterion.bench_function("compiled-next/zoned", |bencher| {
        bencher.iter(|| dense.next_after(black_box(&zoned_cursor)));
    });

    let sparse = Schedule::from_str("0 0 29 2 *").expect("valid schedule");
    criterion.bench_function("compiled-next/leap-day", |bencher| {
        bencher.iter(|| sparse.next_after(black_box(&cursor)));
    });

    let fold = Schedule::from_str("*/15 1 * * *").expect("valid schedule");
    let fold_cursor = civil::datetime(2024, 11, 3, 5, 30, 0, 0)
        .to_zoned_datetime(TimeZone::UTC)
        .expect("benchmark date")
        .with_time_zone(new_york);
    criterion.bench_function("compiled-next/dst-fold", |bencher| {
        bencher.iter(|| fold.next_after(black_box(&fold_cursor)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
