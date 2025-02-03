#![cfg_attr(feature = "bin", no_main)]

extern crate date;
extern crate entry;

use self::date::DateTime;

const USAGE: &str = "usage: date -u [+FORMAT]";
pub const DESCRIPTION: &str = "Print or set the system date and time";

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut now = DateTime::now(false);
    let mut format = String::from("%a %b %r %H:%M:%S %Z %Y");

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
