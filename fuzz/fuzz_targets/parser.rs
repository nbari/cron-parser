#![no_main]

use cron_parser::Schedule;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(expression) = std::str::from_utf8(data) {
        let _ = expression.parse::<Schedule>();
    }
});
