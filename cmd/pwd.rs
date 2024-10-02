#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;

const USAGE: &str = "usage: pwd [-L|-P]";

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut physical = false;

    argument! {
        args: args,
        options: {
            L => physical = false,
            P => physical = true
        },
        command: |_| usage!(),
        on_invalid: |arg| usage!("pwd: invalid option -- '{}'", arg as char)
    }

    let path = if physical {
        std::env::current_dir().unwrap_or_else(|e| error!("pwd: {}", e))
    } else {
        std::env::var("PWD")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|e| error!("pwd: {}", e)))
    };

    println!("{}", path.display());
}
