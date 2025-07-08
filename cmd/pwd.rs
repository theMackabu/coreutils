#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

const USAGE: &str = "usage: pwd [-L|-P]";
pub const DESCRIPTION: &str = "Print name of current/working directory";

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut physical = false;

    argument! {
        args,
        name: "pwd",
        flags: {
            L => physical = false,
            P => physical = true
        },
        options: {},
        command: |_| usage!(),
        on_invalid: |arg| usage!("pwd: invalid option -- '{}'", arg as char)
    }

    let path = if physical {
        std::env::current_dir().unwrap_or_else(|e| error!("pwd: {}", e))
    } else {
        std::env::var("PWD")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|e| error!("pwd: {}", e)))
    };

    println!("{}", path.display());
}
