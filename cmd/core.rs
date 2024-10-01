#![feature(start)]

#[macro_use]
extern crate macros;
extern crate prelude;

mod cat;
mod cp;
mod ls;
mod mkdir;
mod mv;
mod rm;
mod touch;
mod wc;

use std::ffi::CStr;

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

#[start]
fn _start(argc: isize, argv: *const *const u8) -> isize {
    let args: Vec<&[u8]> = (1..argc).map(|arg| unsafe { CStr::from_ptr(*argv.offset(arg) as *const i8).to_bytes() }).collect();

    if args.is_empty() {
        usage!();
    }

    match args[0] {
        b"cat" => start!(cat, argc, argv),
        b"cp" => start!(cp, argc, argv),
        b"ls" => start!(ls, argc, argv),
        b"mkdir" => start!(mkdir, argc, argv),
        b"mv" => start!(mv, argc, argv),
        b"rm" => start!(rm, argc, argv),
        b"touch" => start!(touch, argc, argv),
        b"wc" => start!(wc, argc, argv),
        _ => {
            eprintln!("core: '{}' is not a core command. See 'core --help'.", String::from_utf8_lossy(args[0]));
            1
        }
    }
}
