#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

const USAGE: &str = "usage: hostid";
pub const DESCRIPTION: &str = "Print the numeric identifier for the current host";

#[link(name = "c")]
extern "C" {
    fn gethostid() -> i64;
}

#[entry::gen("bin")]
fn entry() -> ! {
    argument! {
        args,
        flags: {},
        options: {},
        command: |_| usage!(),
        on_invalid: |arg| usage!("hostid: invalid option -- '{}'", arg as char)
    }

    let hostid = gethostid();
    println!("{:08x}", hostid as u32);
}
