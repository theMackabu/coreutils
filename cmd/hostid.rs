#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

const USAGE: &str = "usage: hostid";
pub const DESCRIPTION: &str = "Print the numeric identifier for the current host";

#[link(name = "c")]
extern "C" {
    fn gethostid() -> i64;
}

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    argument! {
        args: args,
        options: {},
        command: |_| usage!(),
        on_invalid: |arg| usage!("hostid: invalid option -- '{}'", arg as char)
    }

    let hostid = unsafe { gethostid() };
    println!("{:08x}", hostid as u32);
}
