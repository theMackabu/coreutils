#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;
extern crate uid;

#[cfg(feature = "bin")]
extern crate prelude;

use self::uid::*;
use prelude::*;

const USAGE: &str = "usage: chown [-h] OWNER[:GROUP] FILE...";
pub const COMMAND: (&str, &str) = ("chown", "Change file owner and group");

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut no_dereference = false;
    let mut owner_group = None;
    let mut files = Vec::new();

    argument! {
        args: args,
        options: {
            h => no_dereference = true
        },
        command: |arg| {
            if owner_group.is_none() {
                owner_group = Some(String::from_utf8_lossy(arg).into_owned());
            } else {
                files.push(PathBuf::from(OsStr::from_bytes(arg)));
            }
        },
        on_invalid: |arg| usage!("chown: invalid option -- '{}'", arg as char)
    }

    let binding = owner_group.to_owned().unwrap_or_else(|| usage!("chown: missing owner[:group]"));

    let lbinding: &'static str = unsafe {
        let ptr = binding.as_ptr();
        let len = binding.len();
        std::mem::forget(binding.to_owned());
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
    };

    let (owner, group) = binding.split_once(':').unwrap_or_else(|| (lbinding, ""));

    let uid = match owner.parse::<u32>() {
        Ok(id) => id,
        Err(_) => get_user_info(Some(owner), None).unwrap_or_else(|_| error!("chown: invalid owner: '{}'", owner)).0,
    };

    let gid = if !group.is_empty() {
        match group.parse::<u32>() {
            Ok(id) => Some(id),
            Err(_) => Some(get_group_id(group).unwrap_or_else(|_| error!("chown: invalid group: '{}'", group))),
        }
    } else {
        None
    };

    for file in files {
        let result = if no_dereference {
            std::os::unix::fs::lchown(&file, Some(uid), gid)
        } else {
            std::os::unix::fs::chown(&file, Some(uid), gid)
        };

        if let Err(e) = result {
            error!("chown: changing ownership of '{}': {}", file.display(), e);
        }
    }
}
