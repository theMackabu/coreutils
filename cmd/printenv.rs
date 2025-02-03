#![cfg_attr(feature = "bin", no_main)]

extern crate entry;
extern crate env;

use std::{ffi::OsStr, os::unix::prelude::OsStrExt};

pub const DESCRIPTION: &str = "Print the environment";

#[entry::gen("bin", "no_iter", "safe")]
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
