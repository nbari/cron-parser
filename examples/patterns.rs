use chrono::Utc;
use cron_parser::parse;

fn main() {
    let title = "Common Cron Expression Patterns";
    println!("{title}");
    println!("{}", "-".repeat(title.len()));
    println!();

    let now = Utc::now();
    println!("Current time: {}", now.format("%Y-%m-%d %H:%M:%S %Z"));
    println!();

    let patterns = vec![
        ("* * * * *", "Every minute"),
        ("*/5 * * * *", "Every 5 minutes"),
        ("*/15 * * * *", "Every 15 minutes"),
        ("0 * * * *", "Every hour (at minute 0)"),
        ("0 */2 * * *", "Every 2 hours"),
        ("0 0 * * *", "Daily at midnight"),
        ("0 2 * * *", "Daily at 2:00 AM"),
        ("0 9 * * 1-5", "Weekdays at 9:00 AM"),
        ("0 0 * * 0", "Weekly on Sunday at midnight"),
        ("0 0 1 * *", "Monthly on the 1st at midnight"),
        ("0 0 1 1 *", "Yearly on January 1st at midnight"),
        ("0 9,17 * * *", "Daily at 9:00 AM and 5:00 PM"),
        ("30 9 * * Mon-Fri", "Weekdays at 9:30 AM"),
        ("0 12-18/2 * * *", "Every 2 hours between 12:00 and 18:00"),
        (
            "0 12-18/3 * * *",
            "Every 3 hours between 12:00 and 18:00 (12:00, 15:00, 18:00)",
        ),
        (
            "*/10 9-17 * * 1-5",
            "Every 10 minutes during business hours (9-5, Mon-Fri)",
        ),
        ("0 0 */3 * *", "Every 3 days at midnight"),
        ("0 0,12 * * *", "Twice a day at midnight and noon"),
        ("15 2 * * 6", "Every Saturday at 2:15 AM"),
    ];

    for (pattern, description) in patterns {
        print_pattern(&now, pattern, description);
    }

    println!();
    println!("Tip: Use 'just run-example \"<pattern>\" --count 10' to see more occurrences");
}

fn print_pattern(now: &chrono::DateTime<chrono::Utc>, pattern: &str, description: &str) {
    match parse(pattern, now) {
        Ok(next) => {
            println!("{description:<30} {pattern}");
            println!("  Next: {}", next.format("%Y-%m-%d %H:%M:%S %Z"));
            println!();
        }
        Err(e) => {
            println!("{description:<30} {pattern}");
            println!("  Error: {e:?}");
            println!();
        }
    }
}
