#![cfg_attr(feature = "bin", feature(start))]

extern crate entry;

const USAGE: &str = "usage: readlink [-f] FILE";
pub const DESCRIPTION: &str = "Print resolved symbolic links or canonical file names";

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut follow = false;
    let mut file = None;

    argument! {
        args: args,
        options: {
            f => follow = true
        },
        command: |arg| {
            if file.is_some() {
                usage!("readlink: too many arguments");
            }
            file = Some(PathBuf::from(OsStr::from_bytes(arg)))
        },
        on_invalid: |arg| usage!("readlink: invalid option -- '{}'", arg as char)
    }

    let path = file.unwrap_or_else(|| usage!("readlink: missing file operand"));

    let result = if follow {
        fs::canonicalize(&path)
    } else {
        fs::read_link(&path)
    };

    match result {
        Ok(resolved_path) => println!("{}", resolved_path.display()),
        Err(e) => error!("readlink: {}: {}", path.display(), e),
    }
}
