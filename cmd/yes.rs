#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use std::io::{self, Write};

pub const COMMAND: (&str, &str) = ("yes", "Print a string repeatedly");

#[entry::gen(cfg = ["bin", "no_ret", "no_iter", "mut"])]
fn entry() -> ! {
    if args.is_empty() {
        args.push(b"y");
    }

    let output = args.join(&b" "[..]);
    let mut stdout = io::stdout();

    loop {
        if let Err(err) = writeln!(stdout, "{}", std::str::from_utf8(&output).unwrap_or("?")) {
            if err.kind() == io::ErrorKind::BrokenPipe {
                std::process::exit(0);
            } else {
                eprintln!("yes: error: {err}");
                std::process::exit(1);
            }
        }
    }
}
