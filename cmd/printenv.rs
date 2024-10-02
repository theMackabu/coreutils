#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;
extern crate env;

#[cfg(feature = "bin")]
extern crate prelude;

use std::{ffi::OsStr, os::unix::prelude::OsStrExt};

#[entry::gen(cfg = ["bin", "no_iter"])]
fn entry() -> ! {
    if args.is_empty() {
        env::vars().iter().for_each(|val| println!("{val}"));
    } else {
        for var in args {
            if let Some(val) = env::get(OsStr::from_bytes(var)) {
                println!("{}", val.to_string_lossy());
            }
        }
    }
}
