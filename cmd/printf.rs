#![cfg_attr(feature = "bin", feature(start))]

extern crate entry;

use std::io::{self, Write};
use std::str::from_utf8;

const USAGE: &str = "usage: printf FORMAT [ARGUMENT]...";
pub const DESCRIPTION: &str = "Format and print data";

fn parse_format(fmt: &str, args: &[&str]) -> String {
    let mut result = String::new();
    let mut chars = fmt.chars().peekable();
    let mut arg_index = 0;

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                match next {
                    '%' => {
                        result.push('%');
                        chars.next();
                    }
                    's' => {
                        if let Some(arg) = args.get(arg_index) {
                            result.push_str(arg);
                            arg_index += 1;
                        }
                        chars.next();
                    }
                    'd' => {
                        if let Some(arg) = args.get(arg_index) {
                            if let Ok(num) = arg.parse::<i32>() {
                                result.push_str(&num.to_string());
                            } else {
                                result.push_str("0");
                            }
                            arg_index += 1;
                        }
                        chars.next();
                    }
                    _ => result.push(c),
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[entry::gen("bin", "mut", "safe")]
fn entry() -> ! {
    let mut args: Vec<&str> = args.map(|arg| from_utf8(arg).unwrap_or("?")).collect();

    if args.is_empty() {
        usage!();
    }

    let format = args.remove(0);
    let arg_refs: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    let formatted = parse_format(&format, &arg_refs);

    io::stdout().write_all(formatted.as_bytes()).expect("expected to write to stdout");
    io::stdout().flush().expect("expected to flush stdout");
}
