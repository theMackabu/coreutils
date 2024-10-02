#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;

const USAGE: &str = "usage: ln [-s] [-f] TARGET LINK_NAME";
pub const COMMAND: (&str, &str) = ("ln", "Make links between files");

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

    let result = if symbolic {
        std::os::unix::fs::symlink(&target, &link_name)
    } else {
        std::fs::hard_link(&target, &link_name)
    };

    if let Err(err) = result {
        error!("ln: {err}")
    }
}
