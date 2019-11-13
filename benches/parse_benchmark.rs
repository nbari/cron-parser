use chrono::Utc;
use criterion::{criterion_group, criterion_main, Criterion};
use cron_parser::parse;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse", |b| b.iter(|| parse("0 0 * * Wed-Fri", Utc::now())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
