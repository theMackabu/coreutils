#![feature(start)]

#[macro_use]
extern crate macros;
extern crate entry;
extern crate prelude;

mod cat;
mod cp;
mod ls;
mod mkdir;
mod mv;
mod rm;
mod touch;
mod wc;

use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path, str};

struct Args {
    argc: isize,
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
    wc      print newline, word, and byte counts for each file";

#[entry::gen]
fn entry() -> ! {
    let path = Path::new(OsStr::from_bytes(program));
    let program = path.file_name().map(|s| s.as_bytes()).unwrap_or(b"core");

    let mut s_arg = Args { argc, program, argv, caller: program };

    if s_arg.program == b"core" {
        if args.is_empty() {
            usage!();
        }
        s_arg.program = args[0];
    }

    match s_arg.program {
        b"cat" => start!(cat, s_arg),
        b"cp" => start!(cp, s_arg),
        b"ls" => start!(ls, s_arg),
        b"mkdir" => start!(mkdir, s_arg),
        b"mv" => start!(mv, s_arg),
        b"rm" => start!(rm, s_arg),
        b"touch" => start!(touch, s_arg),
        b"wc" => start!(wc, s_arg),
        fallback => {
            let cmd = str::from_utf8(fallback).unwrap_or("?");
            error!("core: '{cmd}' is not a core command. See 'core --help'.")
        }
    }
}
