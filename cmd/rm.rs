#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;

const USAGE: &str = "usage: rm [-rf] file...";
pub const DESCRIPTION: &str = "Remove files or directories";

struct RemoveOptions {
    recursive: bool,
    force: bool,
}

fn remove_file(path: &Path, options: &RemoveOptions) -> Result<(), Box<dyn Error>> {
    if !options.force && !path.exists() {
        return Err(format!("rm: {}: No such file or directory", path.display()).into());
    }

    if path.is_dir() {
        if !options.recursive {
            return Err(format!("rm: {}: is a directory", path.display()).into());
        }
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

#[entry::gen(cfg = ["bin", "mut"])]
fn entry() -> ! {
    let mut options = RemoveOptions { recursive: false, force: false };
    let mut files = Vec::new();

    if argc < 2 {
        usage!();
    }

    while let Some(arg) = args.next() {
        match arg {
            b"-r" | b"-R" => options.recursive = true,
            b"-f" => options.force = true,
            b"-rf" | b"-fR" | b"-Rf" | b"-fr" => {
                options.recursive = true;
                options.force = true;
            }
            _ => files.push(OsStr::from_bytes(arg)),
        }
    }

    if files.is_empty() {
        error!("rm: missing operand");
    }

    for file in files {
        let path = Path::new(file);
        if let Err(err) = remove_file(path, &options) {
            if !options.force {
                error!("{}", err);
            }
        }
    }
}
