use chrono::Utc;
use chrono_tz::{America::New_York, Asia::Tokyo, Europe::London, US::Pacific};
use cron_parser::parse;

fn main() {
    let cron_expr = "0 9 * * 1-5"; // Every weekday at 9:00 AM

    println!("Cron expression: {cron_expr} (Every weekday at 9:00 AM)");
    println!();

    // Get current UTC time
    let utc_now = Utc::now();
    println!(
        "Current UTC time: {}",
        utc_now.format("%Y-%m-%d %H:%M:%S %Z")
    );
    println!();

    println!("Next execution time in different timezones:");
    println!("--------------------------------------------------------");

    // UTC
    print_next("UTC", cron_expr, &utc_now, false);

    // Pacific
    let pacific_now = utc_now.with_timezone(&Pacific);
    print_next("US/Pacific", cron_expr, &pacific_now, true);

    // New York
    let ny_now = utc_now.with_timezone(&New_York);
    print_next("America/New_York", cron_expr, &ny_now, true);

    // London
    let london_now = utc_now.with_timezone(&London);
    print_next("Europe/London", cron_expr, &london_now, true);

    // Tokyo
    let tokyo_now = utc_now.with_timezone(&Tokyo);
    print_next("Asia/Tokyo", cron_expr, &tokyo_now, true);

    println!();
    println!("Note: The same cron expression produces different absolute times");
    println!("depending on the timezone, but represents the same local time.");
}

fn print_next<TZ: chrono::TimeZone>(
    label: &str,
    cron_expr: &str,
    now: &chrono::DateTime<TZ>,
    also_print_utc: bool,
) where
    TZ::Offset: std::fmt::Display,
{
    match parse(cron_expr, now) {
        Ok(next) => {
            println!("{label:<16} {}", next.format("%Y-%m-%d %H:%M:%S %Z"));
            if also_print_utc {
                println!(
                    "{:16} (UTC: {})",
                    "",
                    next.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S %Z")
                );
            }
        }
        Err(e) => println!("{label:<16} Error: {e:?}"),
    }
}
