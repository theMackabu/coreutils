#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

const USAGE: &str = "usage: echo [-n] [STRING]...";
pub const DESCRIPTION: &str = "Display a line of text";

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut has_newline = true;
    let mut first_arg = true;

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    argument! {
        args,
        name: "echo",
        flags: {
            n => has_newline = false,
            h => usage!(help->$)
        },
        options: {},
        command: |arg| {
            if !first_arg {
                let _ = handle.write_all(b" ");
            }
            let _ = handle.write_all(arg);
            first_arg = false;
        },
        on_invalid: |arg| usage!("echo: invalid option -- '{arg}'")
    }

    if has_newline {
        let _ = handle.write_all(b"\n");
    }
    let _ = handle.flush();
}
