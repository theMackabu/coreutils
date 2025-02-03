#![cfg_attr(feature = "bin", no_main, feature(rustc_private))]

extern crate entry;

use std::os::unix::io::AsRawFd;

const USAGE: &str = "usage: tty [-s]";
pub const DESCRIPTION: &str = "Print the file name of the terminal";

#[cfg(target_os = "macos")]
#[link(name = "c")]
extern "C" {
    fn ttyname(fd: i32) -> *const libc::c_char;
    fn isatty(fd: i32) -> i32;
}

#[cfg(not(target_os = "macos"))]
#[link(name = "c")]
extern "C" {
    fn ttyname(fd: i32) -> *const libc::c_char;
    fn isatty(fd: i32) -> i32;
}

fn get_tty_name() -> Option<String> {
    let fd = std::io::stdin().as_raw_fd();
    unsafe {
        if isatty(fd) != 1 {
            return None;
        }
        let ptr = ttyname(fd);
        if ptr.is_null() {
            None
        } else {
            Some(std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }
}

#[entry::gen("bin", "no_ret", "safe", "libc")]
fn entry() -> ! {
    let mut silent = false;

    argument! {
        args: args,
        options: {
            s => silent = true
        },
        command: |_| usage!(),
        on_invalid: |arg| usage!("tty: invalid option -- '{}'", arg as char)
    }

    match get_tty_name() {
        Some(tty_name) => {
            if !silent {
                println!("{tty_name}");
            }
            std::process::exit(0);
        }
        None => {
            if !silent {
                println!("not a tty");
            }
            std::process::exit(1);
        }
    }
}
