#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use std::thread;
use std::time::Duration;

const USAGE: &str = "usage: sleep NUMBER[msMhd]";
pub const COMMAND: (&str, &str) = ("sleep", "Delay for a specified amount of time");

fn parse_duration(s: &str) -> Result<Duration, Box<dyn std::error::Error>> {
    let mut chars = s.chars().peekable();
    let mut num_str = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_digit(10) || c == '.' {
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    let multiplier = match chars.next() {
        Some('s') | None => 1.0,
        Some('m') => 60.0,
        Some('h') => 3600.0,
        Some('d') => 86400.0,
        err => return Err(err.expect("?").to_string().into()),
    };

    Ok(Duration::from_secs_f64(num_str.parse::<f64>()? * multiplier))
}

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut total_duration = Duration::new(0, 0);

    argument! {
        args: args,
        options: {},
        command: |arg| {
            match parse_duration(&String::from_utf8_lossy(arg)) {
                Ok(duration) => total_duration += duration,
                Err(e) => error!("sleep: invalid time interval '{}'", e),
            }
        },
        on_invalid: |arg| usage!("sleep: invalid option -- '{}'", arg as char)
    }

    if total_duration.as_secs() == 0 && total_duration.subsec_nanos() == 0 {
        usage!();
    }

    thread::sleep(total_duration);
}
