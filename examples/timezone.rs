use cron_parser::parse;
use jiff::{Timestamp, ZonedDateTime, tz::TimeZone};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cron_expr = "0 9 * * 1-5"; // Every weekday at 9:00 AM

    println!("Cron expression: {cron_expr} (Every weekday at 9:00 AM)");
    println!();

    // Get current UTC time
    let utc_now = Timestamp::now().to_zoned_datetime(TimeZone::UTC);
    println!(
        "Current UTC time: {}",
        utc_now.strftime("%Y-%m-%d %H:%M:%S %Z")
    );
    println!();

    println!("Next execution time in different timezones:");
    println!("--------------------------------------------------------");

    // UTC
    print_next("UTC", cron_expr, &utc_now, false);

    // Pacific
    let pacific_now = utc_now.in_tz("America/Los_Angeles")?;
    print_next("America/Los_Angeles", cron_expr, &pacific_now, true);

    // New York
    let ny_now = utc_now.in_tz("America/New_York")?;
    print_next("America/New_York", cron_expr, &ny_now, true);

    // London
    let london_now = utc_now.in_tz("Europe/London")?;
    print_next("Europe/London", cron_expr, &london_now, true);

    // Tokyo
    let tokyo_now = utc_now.in_tz("Asia/Tokyo")?;
    print_next("Asia/Tokyo", cron_expr, &tokyo_now, true);

    println!();
    println!("Note: The same cron expression produces different absolute times");
    println!("depending on the timezone, but represents the same local time.");
    Ok(())
}

fn print_next(label: &str, cron_expr: &str, now: &ZonedDateTime, also_print_utc: bool) {
    match parse(cron_expr, now) {
        Ok(next) => {
            println!("{label:<16} {}", next.strftime("%Y-%m-%d %H:%M:%S %Z"));
            if also_print_utc {
                println!(
                    "{:16} (UTC: {})",
                    "",
                    next.with_time_zone(TimeZone::UTC)
                        .strftime("%Y-%m-%d %H:%M:%S %Z")
                );
            }
        }
        Err(e) => println!("{label:<16} Error: {e:?}"),
    }
}
