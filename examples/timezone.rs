use chrono::Utc;
use chrono_tz::{America::New_York, Asia::Tokyo, Europe::London, US::Pacific};
use cron_parser::parse;

fn main() {
    let cron_expr = "0 9 * * 1-5"; // Every weekday at 9:00 AM

    println!("Cron expression: {} (Every weekday at 9:00 AM)", cron_expr);
    println!();

    // Get current UTC time
    let utc_now = Utc::now();
    println!(
        "Current UTC time: {}",
        utc_now.format("%Y-%m-%d %H:%M:%S %Z")
    );
    println!();

    println!("Next execution time in different timezones:");
    println!("");

    // UTC
    if let Ok(next) = parse(cron_expr, &utc_now) {
        println!("UTC:              {}", next.format("%Y-%m-%d %H:%M:%S %Z"));
    }

    // Pacific
    let pacific_now = utc_now.with_timezone(&Pacific);
    if let Ok(next) = parse(cron_expr, &pacific_now) {
        println!("US/Pacific:       {}", next.format("%Y-%m-%d %H:%M:%S %Z"));
        println!(
            "                  (UTC: {})",
            next.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S %Z")
        );
    }

    // New York
    let ny_now = utc_now.with_timezone(&New_York);
    if let Ok(next) = parse(cron_expr, &ny_now) {
        println!("America/New_York: {}", next.format("%Y-%m-%d %H:%M:%S %Z"));
        println!(
            "                  (UTC: {})",
            next.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S %Z")
        );
    }

    // London
    let london_now = utc_now.with_timezone(&London);
    if let Ok(next) = parse(cron_expr, &london_now) {
        println!("Europe/London:    {}", next.format("%Y-%m-%d %H:%M:%S %Z"));
        println!(
            "                  (UTC: {})",
            next.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S %Z")
        );
    }

    // Tokyo
    let tokyo_now = utc_now.with_timezone(&Tokyo);
    if let Ok(next) = parse(cron_expr, &tokyo_now) {
        println!("Asia/Tokyo:       {}", next.format("%Y-%m-%d %H:%M:%S %Z"));
        println!(
            "                  (UTC: {})",
            next.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S %Z")
        );
    }

    println!();
    println!("Note: The same cron expression produces different absolute times");
    println!("depending on the timezone, but represents the same local time.");
}
