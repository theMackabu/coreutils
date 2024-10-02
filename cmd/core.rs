#![feature(start)]

#[macro_use]
extern crate macros;
extern crate entry;
extern crate prelude;

mod cat;
mod cp;
mod du;
mod ls;
mod mkdir;
mod mv;
mod rm;
mod touch;
mod wc;
mod yes;

use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path, str};

struct Args {
    argc: isize,
    args: Vec<&'static [u8]>,
    caller: &'static [u8],
    program: &'static [u8],
    argv: *const *const u8,
}

const USAGE: &str = "usage: core <command> [arguments...]

Available commands:
    cat     concatenate and print files
    cp      copy files and directories
    ls      list directory contents
    mkdir   make directories
    mv      move (rename) files
    rm      remove files or directories
    touch   change file timestamps
    du      estimate file space usage
    yes     Print a string repeatedly
    wc      print newline, word, and byte counts for each file";

#[entry::gen]
fn entry() -> ! {
    let path = Path::new(OsStr::from_bytes(program));
    let program = path.file_name().map(|s| s.as_bytes()).unwrap_or(b"core");

    entry! {
        args: { argc, args, program, argv, caller: program },
        commands: [cat, cp, ls, mkdir, mv, du, rm, touch, yes, wc],
        fallback: |cmd| {
            match cmd {
                "--help" => usage!(),
                "--version" => stdout!("{} ({} {})", env!("PKG_VERSION"), env!("BUILD_DATE"), env!("GIT_HASH")),
                _ => error!("core: '{cmd}' is not a core command. See 'core --help'.")
            }
        }
    }
}
