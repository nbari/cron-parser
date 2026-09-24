use chrono::Utc;
use cron_parser::Schedule;
use std::env;

fn main() {
    let mut args = env::args();
    let bin_name = args.next().unwrap_or_else(|| "parse".to_string());

    let Some(cron_expr) = args.next() else {
        print_usage_and_exit(&bin_name);
    };
    if cron_expr == "--help" || cron_expr == "-h" {
        print_usage_and_exit(&bin_name);
    }

    let mut count: usize = 5;
    let mut rest = args.peekable();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--count" | "-n" => {
                let Some(value) = rest.next() else {
                    eprintln!("Missing value for {arg}");
                    print_usage_and_exit(&bin_name);
                };
                count = value.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("Invalid value for {arg}: {value}");
                    print_usage_and_exit(&bin_name);
                });
            }
            "--help" | "-h" => print_usage_and_exit(&bin_name),
            _ => {
                eprintln!("Unknown argument: {arg}");
                print_usage_and_exit(&bin_name);
            }
        }
    }

    // Get current time
    let now = Utc::now();

    println!("Cron expression: {cron_expr}");
    println!("Current time:    {}", now.format("%Y-%m-%d %H:%M:%S %Z"));
    println!();

    // Compile once, then reuse the schedule for every occurrence.
    match cron_expr.parse::<Schedule>() {
        Ok(schedule) => {
            println!("Next {count} execution times:");
            println!("-----------------------------------------------------");

            for (index, next) in schedule.after(&now).take(count).enumerate() {
                let i = index + 1;
                println!("{:2}. {}", i, next.format("%Y-%m-%d %H:%M:%S %Z"));
            }
        }
        Err(e) => {
            eprintln!("Error parsing cron expression: {e:?}");
            eprintln!();
            eprintln!("Common issues:");
            eprintln!("  - Invalid field values (e.g., minute > 59)");
            eprintln!("  - Invalid range (e.g., 10-5)");
            eprintln!("  - Invalid step (e.g., */0)");
            eprintln!("  - Wrong number of fields (must be exactly 5)");
            std::process::exit(1);
        }
    }
}

fn print_usage_and_exit(bin_name: &str) -> ! {
    eprintln!("Usage: {bin_name} <cron-expression> [--count <n>]");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {bin_name} \"*/5 * * * *\"");
    eprintln!("  {bin_name} \"0 0 * * *\"");
    eprintln!("  {bin_name} \"0 9 * * 1-5\" --count 10");
    eprintln!();
    eprintln!("Cron expression format:");
    eprintln!("  ┌─────────────────────  minute (0 - 59)");
    eprintln!("  │ ┌───────────────────  hour   (0 - 23)");
    eprintln!("  │ │ ┌─────────────────  dom    (1 - 31) day of month");
    eprintln!("  │ │ │ ┌───────────────  month  (1 - 12)");
    eprintln!("  │ │ │ │ ┌─────────────  dow    (0 - 6 or Sun - Sat) day of week");
    eprintln!("  │ │ │ │ │");
    eprintln!("  * * * * *");
    std::process::exit(1);
}
