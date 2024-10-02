#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate date;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use self::date::DateTime;

const USAGE: &str = "usage: date -u [+FORMAT]";
pub const COMMAND: (&str, &str) = ("date", "Print or set the system date and time");

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut now = DateTime::now(false);
    let mut format = String::from("%a %b %d %H:%M:%S %Z %Y");

    argument! {
        args: args,
        options: {
            u => now = DateTime::now(true)
        },
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

    println!("{}", now.format(&format));
}
