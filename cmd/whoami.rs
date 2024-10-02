#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;
use std::ffi::CStr;
use std::os::raw::c_char;

const USAGE: &str = "usage: whoami";
pub const DESCRIPTION: &str = "Print effective user name";

extern "C" {
    fn getlogin() -> *const c_char;
    fn getpwuid(uid: u32) -> *const passwd;
    fn geteuid() -> u32;
}

#[repr(C)]
struct passwd {
    pw_name: *const c_char,
    pw_passwd: *const c_char,
    pw_uid: u32,
    pw_gid: u32,
    pw_gecos: *const c_char,
    pw_dir: *const c_char,
    pw_shell: *const c_char,
}

fn get_username() -> Result<String, Box<dyn Error>> {
    unsafe {
        let login = getlogin();
        if !login.is_null() {
            return Ok(CStr::from_ptr(login).to_string_lossy().into_owned());
        }

        let uid = geteuid();
        let pw = getpwuid(uid);
        if pw.is_null() {
            return Err("Failed to get user information".into());
        }

        let username = CStr::from_ptr((*pw).pw_name);
        Ok(username.to_string_lossy().into_owned())
    }
}

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    if args.len() > 1 {
        usage!();
    }

    match get_username() {
        Ok(username) => println!("{}", username),
        Err(e) => error!("whoami: {}", e),
    }
}
