#![feature(start)]

#[macro_use]
extern crate macros;

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
        b"cat" => cat::_start(argc - 1, unsafe { argv.offset(1) }),
        b"cp" => cp::_start(argc - 1, unsafe { argv.offset(1) }),
        b"ls" => ls::_start(argc - 1, unsafe { argv.offset(1) }),
        b"mkdir" => mkdir::_start(argc - 1, unsafe { argv.offset(1) }),
        b"mv" => mv::_start(argc - 1, unsafe { argv.offset(1) }),
        b"rm" => rm::_start(argc - 1, unsafe { argv.offset(1) }),
        b"touch" => touch::_start(argc - 1, unsafe { argv.offset(1) }),
        b"wc" => wc::_start(argc - 1, unsafe { argv.offset(1) }),
        _ => {
            eprintln!("core: '{}' is not a core command. See 'core --help'.", String::from_utf8_lossy(args[0]));
            1
        }
    }
}
