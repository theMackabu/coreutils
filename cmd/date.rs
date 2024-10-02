#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use std::time::{SystemTime, UNIX_EPOCH};

const USAGE: &str = "usage: date [+FORMAT]";
pub const COMMAND: (&str, &str) = ("date", "Print or set the system date and time");

fn format_date(format: &str) -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_else(|e| e.duration());

    let secs = duration.as_secs() as i64;
    let nsecs = duration.subsec_nanos();

    let mut year = 1970 + (secs / 31_536_000);
    let mut remaining_secs = secs % 31_536_000;

    let mut month = 1;
    let mut days_in_months = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    if is_leap_year {
        days_in_months[1] = 29;
    }

    while month <= 12 {
        let days_this_month = days_in_months[month as usize - 1];
        if remaining_secs < days_this_month * 86_400 {
            break;
        }
        remaining_secs -= days_this_month * 86_400;
        month += 1;
    }

    let day = (remaining_secs / 86_400) + 1;
    remaining_secs %= 86_400;

    let hour_24 = (remaining_secs / 3600) as i64;
    let hour_12 = if hour_24 == 0 { 12 } else { hour_24 % 12 };
    let minute = (remaining_secs % 3600) / 60;
    let second = remaining_secs % 60;

    let day_of_year: i64 = days_in_months.iter().take((month - 1) as usize).sum::<i64>() + day as i64;

    let (mut m, mut y) = (month, year);
    if m < 3 {
        m += 12;
        y -= 1;
    }

    let k = y % 100;
    let j = y / 100;
    let weekday = (day + (13 * (m + 1)) / 5 + k + (k / 4) + (j / 4) - (2 * j)) % 7;
    let weekday = (weekday + 6) % 7 + 1;

    let mut result = String::new();
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('D') => result.push_str(&format!("{:02}/{:02}/{:02}", month, day, year % 100)),
                Some('Y') => result.push_str(&format!("{:04}", year)),
                Some('m') => result.push_str(&format!("{:02}", month)),
                Some('B') => result.push_str(&format!(
                    "{}",
                    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][month as usize - 1]
                )),
                Some('b') => result.push_str(&format!("{}", ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][month as usize - 1])),
                Some('d') => result.push_str(&format!("{:02}", day)),
                Some('j') => result.push_str(&format!("{:03}", day_of_year)),
                Some('u') => result.push_str(&format!("{}", weekday)),
                Some('A') => result.push_str(&format!("{}", ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"][weekday as usize - 1])),
                Some('a') => result.push_str(&format!("{}", ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][weekday as usize - 1])),
                Some('H') => result.push_str(&format!("{:02}", hour_24)),
                Some('I') => result.push_str(&format!("{:02}", hour_12)),
                Some('M') => result.push_str(&format!("{:02}", minute)),
                Some('S') => result.push_str(&format!("{:02}", second)),
                Some('s') => result.push_str(&format!("{}", secs)),
                Some('N') => result.push_str(&format!("{:09}", nsecs)),
                Some('%') => result.push('%'),
                Some(x) => result.push_str(&format!("%{}", x)),
                None => result.push('%'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut format = String::from("%a %b %d %H:%M:%S %Z %Y");

    argument! {
        args: args,
        options: {},
        command: |arg| {
            let arg = String::from_utf8_lossy(arg).into_owned();
            if arg.starts_with('+') {
                format = arg[1..].to_string();
            } else {
                usage!("date: invalid date '{}'", arg);
            }
        },
        on_invalid: |arg| usage!("date: invalid option -- '{}'", arg as char)
    }

    println!("{}", format_date(&format));
}
