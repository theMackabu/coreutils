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
