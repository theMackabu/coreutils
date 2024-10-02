#![feature(start)]

#[macro_use]
extern crate macros;
extern crate entry;
extern crate prelude;

mod cat;
mod cp;
mod du;
mod echo;
mod env;
mod ln;
mod ls;
mod mkdir;
mod mv;
mod printenv;
mod printf;
mod pwd;
mod readlink;
mod rm;
mod stat;
mod sum;
mod tail;
mod touch;
mod tty;
mod uname;
mod wc;
mod who;
mod whoami;
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
    cat       concatenate and print files
    cp        copy files and directories
    ls        list directory contents
    echo      display a line of text
    env       Set the environment for command invocation
    ln        Make links between files
    printenv  Print the environment
    printf    Format and print data
    pwd       Print name of current/working directory
    readlink  Print resolved symbolic links or canonical file names
    mkdir     make directories
    mv        move (rename) files
    rm        remove files or directories
    stat      Display file or file system status
    sum       Checksum and count the blocks in a file
    tail      Output the last part of files
    tty       Print the file name of the terminal
    touch     change file timestamps
    du        estimate file space usage
    uname     Print system information
    yes       Print a string repeatedly
    whoami    Print effective user name
    who       Show who is logged on
    wc        print newline, word, and byte counts for each file";

#[entry::gen]
fn entry() -> ! {
    let path = Path::new(OsStr::from_bytes(program));
    let program = path.file_name().map(|s| s.as_bytes()).unwrap_or(b"core");

    entry! {
        args: { argc, args, program, argv, caller: program },
        commands: [cat, cp, ln, ls, mkdir, mv, du, env, echo, rm, readlink, pwd, printf, printenv, sum, stat, tty, touch, yes, uname, wc, whoami, who],
        fallback: |cmd| {
            match cmd {
                "--help" => usage!(),
                "--version" => stdout!("{} ({} {})", env!("PKG_VERSION"), env!("BUILD_DATE"), env!("GIT_HASH")),
                _ => error!("core: '{cmd}' is not a core command. See 'core --help'.")
            }
        }
    }
}
