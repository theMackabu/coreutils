#![cfg_attr(feature = "start", feature(start))]

#[cfg(feature = "start")]
#[macro_use]
extern crate macros;

use std::{
    error::Error,
    ffi::{CStr, OsStr},
    fs,
    os::unix::ffi::OsStrExt,
    path::Path,
};

const USAGE: &str = "usage: rm [-rf] file...";

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

#[cfg_attr(feature = "start", start)]
pub fn _start(argc: isize, argv: *const *const u8) -> isize {
    let args = (1..argc).map(|arg| unsafe { CStr::from_ptr(*argv.offset(arg) as *const i8).to_bytes() });
    let mut options = RemoveOptions { recursive: false, force: false };
    let mut files = Vec::new();
    let mut args = args.collect::<Vec<&[u8]>>().into_iter();

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

    return 0;
}
