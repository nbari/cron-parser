use chrono::{DateTime, Utc};
use criterion::{criterion_group, criterion_main, Criterion};
use cron_parser::parse;
use core::str::FromStr;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse", |b| {
        b.iter(|| parse("0 0 * * Wed-Fri", &DateTime::<Utc>::from_str("2021-12-02T14:02:29+0000").unwrap()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
