use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use cron_parser::parse;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse", |b| {
        b.iter(|| parse("0 0 * * Wed-Fri", &Utc::now()));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
