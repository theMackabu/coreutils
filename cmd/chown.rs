#![allow(non_camel_case_types)]
#![cfg_attr(feature = "bin", feature(start))]

extern crate entry;
extern crate env;
extern crate uid;

use self::env::{cvt, run_path_with_cstr};
use self::uid::*;
use std::ffi::{c_char, c_int};

const USAGE: &str = "usage: chown [-h] OWNER[:GROUP] FILE...";
pub const DESCRIPTION: &str = "Change file owner and group";

pub type gid_t = u32;
pub type uid_t = u32;

#[link(name = "c")]
extern "C" {
    #[cfg_attr(
        all(target_os = "macos", target_arch = "x86"),
        link_name = "lchown$UNIX2003"
    )]
    fn lchown(path: *const c_char, uid: uid_t, gid: gid_t) -> c_int;
    fn chown(path: *const c_char, uid: uid_t, gid: gid_t) -> c_int;
}

unsafe fn do_lchown<P: AsRef<Path>>(path: P, uid: Option<u32>, gid: Option<u32>) -> io::Result<()> {
    let path = path.as_ref();
    let uid = uid.unwrap_or(u32::MAX);
    let gid = gid.unwrap_or(u32::MAX);

    run_path_with_cstr(path, &|path| {
        cvt(lchown(path.as_ptr(), uid as uid_t, gid as gid_t)).map(|_| ())
    })
}

unsafe fn do_chown<P: AsRef<Path>>(dir: P, uid: Option<u32>, gid: Option<u32>) -> io::Result<()> {
    let dir = dir.as_ref();
    let uid = uid.unwrap_or(u32::MAX);
    let gid = gid.unwrap_or(u32::MAX);

    run_path_with_cstr(dir, &|dir| {
        cvt(chown(dir.as_ptr(), uid as uid_t, gid as gid_t)).map(|_| ())
    })
}

#[entry::gen("bin")]
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

    let binding = owner_group
        .to_owned()
        .unwrap_or_else(|| usage!("chown: missing owner[:group]"));

    let lbinding: &'static str = {
        let ptr = binding.as_ptr();
        let len = binding.len();
        std::mem::forget(binding.to_owned());
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
    };

    let (owner, group) = binding.split_once(':').unwrap_or_else(|| (lbinding, ""));

    let uid = match owner.parse::<u32>() {
        Ok(id) => id,
        Err(_) => {
            get_user_info(Some(owner), None)
                .unwrap_or_else(|_| error!("chown: invalid owner: '{}'", owner))
                .0
        }
    };

    let gid = if !group.is_empty() {
        match group.parse::<u32>() {
            Ok(id) => Some(id),
            Err(_) => Some(
                get_group_id(group).unwrap_or_else(|_| error!("chown: invalid group: '{}'", group)),
            ),
        }
    } else {
        None
    };

    for file in files {
        let result = match no_dereference {
            true => do_lchown(&file, Some(uid), gid),
            false => do_chown(&file, Some(uid), gid),
        };

        if let Err(e) = result {
            error!("chown: changing ownership of '{}': {}", file.display(), e);
        }
    }
}
