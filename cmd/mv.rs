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

const USAGE: &str = "usage: mv [-f | -i | -n] [-v] source... destination";

struct MvOptions {
    force: bool,
    interactive: bool,
    no_clobber: bool,
    verbose: bool,
}

impl MvOptions {
    fn new() -> Self {
        Self {
            force: false,
            interactive: false,
            no_clobber: false,
            verbose: false,
        }
    }
}

fn mv(source: &Path, destination: &Path, options: &MvOptions) -> Result<(), Box<dyn Error>> {
    if options.interactive && destination.exists() {
        print!("overwrite '{}'? ", destination.display());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }
    }

    if options.no_clobber && destination.exists() {
        return Ok(());
    }

    if options.force {
        fs::remove_file(destination).ok();
    }

    fs::rename(source, destination)?;

    if options.verbose {
        println!("renamed '{}' -> '{}'", source.display(), destination.display());
    }

    Ok(())
}

#[cfg_attr(feature = "start", start)]
pub fn _start(argc: isize, argv: *const *const u8) -> isize {
    let args = (1..argc).map(|arg| unsafe { CStr::from_ptr(*argv.offset(arg) as *const i8).to_bytes() });
    let mut options = MvOptions::new();
    let mut sources = Vec::new();
    let mut args = args.collect::<Vec<&[u8]>>().into_iter();

    if argc < 3 {
        usage!();
    }

    while let Some(arg) = args.next() {
        match arg {
            b"-f" => options.force = true,
            b"-i" => options.interactive = true,
            b"-n" => options.no_clobber = true,
            b"-v" => options.verbose = true,
            _ => sources.push(OsStr::from_bytes(arg)),
        }
    }

    if sources.len() < 2 {
        usage!();
    }

    let destination = sources.pop().unwrap();
    let destination = Path::new(destination);

    if sources.len() > 1 && !destination.is_dir() {
        error!("mv: target '{}' is not a directory", destination.display());
    }

    for source in sources {
        let source = Path::new(source);
        let dest = if destination.is_dir() {
            destination.join(source.file_name().unwrap())
        } else {
            destination.to_path_buf()
        };

        if let Err(err) = mv(source, &dest, &options) {
            error!("mv: cannot move '{}' to '{}': {}", source.display(), dest.display(), err);
        }
    }

    return 0;
}
