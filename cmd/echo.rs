#![cfg_attr(feature = "bin", feature(start))]

extern crate entry;

const USAGE: &str = "usage: echo [-n] [STRING]...";
pub const DESCRIPTION: &str = "Display a line of text";

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut no_newline = false;
    let mut strings = Vec::new();

    argument! {
        args: args,
        options: {
            n => no_newline = true
        },
        command: |arg| strings.push(String::from_utf8_lossy(arg).into_owned()),
        on_invalid: |arg| usage!("echo: invalid option -- '{}'", arg as char)
    }

    print!("{}", strings.join(" "));

    if !no_newline {
        println!();
    }
}
