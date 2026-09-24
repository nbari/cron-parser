#![no_main]

use chrono::{TimeZone, Utc};
use cron_parser::Schedule;
use libfuzzer_sys::fuzz_target;

const EXPRESSIONS: [&str; 7] = [
    "* * * * *",
    "*/5 * * * *",
    "0 12-18/3 * * Mon-Fri",
    "30 2 * * *",
    "30 1 * * *",
    "0 0 29 2 *",
    "0 0 1 * Mon",
];

fuzz_target!(|data: &[u8]| {
    let Some(selector) = data.first() else {
        return;
    };
    let schedule: Schedule = EXPRESSIONS[usize::from(*selector) % EXPRESSIONS.len()]
        .parse()
        .expect("seed expression");
    let mut bytes = [0_u8; 8];
    for (target, source) in bytes.iter_mut().zip(data.iter().skip(1)) {
        *target = *source;
    }
    let raw = i64::from_le_bytes(bytes);
    let timestamp = 946_684_800 + raw.rem_euclid(4_102_444_800);
    if let Some(cursor) = Utc.timestamp_opt(timestamp, 0).single() {
        if let Some(next) = schedule.next_after(&cursor) {
            assert!(next > cursor);
            assert!(schedule.includes(&next));
        }
        if let Some(previous) = schedule.previous_before(&cursor) {
            assert!(previous < cursor);
            assert!(schedule.includes(&previous));
        }
    }
});
