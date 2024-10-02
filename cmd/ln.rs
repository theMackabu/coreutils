#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;
extern crate env;

#[cfg(feature = "bin")]
extern crate prelude;

use ln::env::{cvt, run_path_with_cstr};
use prelude::*;
use std::ffi::{c_char, c_int};

const USAGE: &str = "usage: ln [-s] [-f] TARGET LINK_NAME";
pub const DESCRIPTION: &str = "Make links between files";

#[link(name = "c")]
extern "C" {
    fn symlink(path1: *const c_char, path2: *const c_char) -> c_int;
}

fn symbolic_link(original: &Path, link: &Path) -> io::Result<()> {
    run_path_with_cstr(original, &|original| {
        run_path_with_cstr(link, &|link| cvt(unsafe { symlink(original.as_ptr(), link.as_ptr()) }).map(|_| ()))
    })
}

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut symbolic = false;
    let mut force = false;
    let mut target = None;
    let mut link_name = None;

    argument! {
        args: args,
        options: {
            s => symbolic = true,
            f => force = true
        },
        command: |arg| {
            if target.is_none() {
                target = Some(PathBuf::from(OsStr::from_bytes(arg)));
            } else if link_name.is_none() {
                link_name = Some(PathBuf::from(OsStr::from_bytes(arg)));
            } else {
                usage!("ln: too many arguments");
            }
        },
        on_invalid: |arg| usage!("ln: invalid option -- '{}'", arg as char)
    }

    let target = target.unwrap_or_else(|| usage!("ln: missing file operand"));
    let link_name = link_name.unwrap_or_else(|| usage!("ln: missing destination file operand after '{}'", target.display()));

    if force {
        let _ = fs::remove_file(&link_name);
    }

    let result = match symbolic {
        true => symbolic_link(&target, &link_name),
        false => std::fs::hard_link(&target, &link_name),
    };

    if let Err(err) = result {
        error!("ln: {err}")
    }
}
