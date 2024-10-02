#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;

const USAGE: &str = "usage: mv [-f | -i | -n] [-v] source... destination";
pub const DESCRIPTION: &str = "Move (rename) files";

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

#[entry::gen(cfg = ["bin", "mut"])]
fn entry() -> ! {
    let mut options = MvOptions::new();
    let mut sources = Vec::new();

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
}
