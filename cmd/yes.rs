#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

use std::io::{self, Write};

pub const DESCRIPTION: &str = "Print a string repeatedly";

#[entry::gen("bin", "no_ret", "no_iter", "mut", "safe")]
fn entry() -> ! {
    if args.is_empty() {
        args.push(b"y");
    }

    let output = args.join(&b" "[..]);
    let mut stdout = io::stdout();

    loop {
        if let Err(err) = writeln!(stdout, "{}", std::str::from_utf8(&output).unwrap_or("?")) {
            if err.kind() == io::ErrorKind::BrokenPipe {
                return 0;
            } else {
                eprintln!("yes: error: {err}");
                return 1;
            }
        }
    }
}
