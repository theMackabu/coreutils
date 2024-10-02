#![feature(start)]

#[macro_use]
extern crate macros;
extern crate entry;
extern crate prelude;

module! {
    cat, cp, du, echo, env, ln, ls,
    mkdir, mv, printenv, printf, pwd,
    readlink, rm, stat, sum, tail, touch,
    tty, uname, wc, who, whoami, yes
}

use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path, str};

struct Args {
    argc: isize,
    args: Vec<&'static [u8]>,
    caller: &'static [u8],
    program: &'static [u8],
    argv: *const *const u8,
}

lazy_lock! {
    static USAGE: String = generate_usage();
    static COMMANDS: &'static [(&'static str, &'static str)] = init_commands();
}

fn generate_usage() -> String {
    let max_name_len = COMMANDS.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    let available_commands = COMMANDS
        .iter()
        .map(|(name, desc)| format!("    {:<width$} {}", name, desc, width = max_name_len + 2))
        .collect::<Vec<_>>()
        .join("\n");

    format!("usage: core <command> [arguments...]\nAvailable commands:\n{}", available_commands)
}

#[entry::gen]
fn entry() -> ! {
    let path = Path::new(OsStr::from_bytes(program));
    let program = path.file_name().map(|s| s.as_bytes()).unwrap_or(b"core");

    entry! {
        args: { argc, args, program, argv, caller: program },
        commands: [cat, cp, ln, ls, mkdir, mv, du, env, echo, rm, readlink, pwd, printf, printenv, sum, stat, tty, tail, touch, yes, uname, wc, whoami, who],
        fallback: |cmd| {
            match cmd {
                "--help" => usage!(),
                "--version" => stdout!("{} ({} {})", env!("PKG_VERSION"), env!("BUILD_DATE"), env!("GIT_HASH")),
                _ => error!("core: '{cmd}' is not a core command. See 'core --help'.")
            }
        }
    }
}
