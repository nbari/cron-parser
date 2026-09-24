#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use chrono::{TimeZone, Utc};
use chrono_tz::America::New_York;
use criterion::{Criterion, criterion_group, criterion_main};
use cron_parser::{Schedule, parse};
use std::{hint::black_box, str::FromStr};

pub fn criterion_benchmark(criterion: &mut Criterion) {
    let cursor = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
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

    let sparse = Schedule::from_str("0 0 29 2 *").expect("valid schedule");
    criterion.bench_function("compiled-next/leap-day", |bencher| {
        bencher.iter(|| sparse.next_after(black_box(&cursor)));
    });

    let fold = Schedule::from_str("*/15 1 * * *").expect("valid schedule");
    let fold_cursor = Utc
        .with_ymd_and_hms(2024, 11, 3, 5, 30, 0)
        .single()
        .expect("benchmark date")
        .with_timezone(&New_York);
    criterion.bench_function("compiled-next/dst-fold", |bencher| {
        bencher.iter(|| fold.next_after(black_box(&fold_cursor)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
