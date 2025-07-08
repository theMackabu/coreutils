#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

const USAGE: &str = "usage: cp [-Rrfipv] source_file target_file";
pub const DESCRIPTION: &str = "Copy files and directories";

struct CpOptions {
    recursive: bool,
    force: bool,
    interactive: bool,
    no_clobber: bool,
    preserve_attributes: bool,
    verbose: bool,
}

impl CpOptions {
    fn new() -> Self {
        Self {
            recursive: false,
            force: false,
            interactive: false,
            no_clobber: false,
            preserve_attributes: false,
            verbose: false,
        }
    }
}

fn cp(source: &Path, destination: &Path, options: &CpOptions) -> Result<(), Box<dyn Error>> {
    if options.interactive && destination.exists() {
        print!("overwrite '{}'? ", destination.display());
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }
    }

    if options.no_clobber && destination.exists() {
        return Ok(());
    }

    if options.force && destination.exists() {
        if let Err(err) = fs::remove_file(destination) {
            error!("cp: warning: failed to remove '{}': {}", destination.display(), err);
        }
    }
    if source.is_dir() {
        if !options.recursive {
            error!("cp: -r not specified; omitting directory '{}'", source.display());
        }
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let new_dest = destination.join(entry.file_name());
            cp(&entry.path(), &new_dest, options)?;
        }
    } else {
        fs::copy(source, destination)?;
    }

    if options.preserve_attributes {
        let metadata = fs::metadata(source)?;
        fs::set_permissions(destination, metadata.permissions())?;
    }

    if options.verbose {
        println!("'{}' -> '{}'", source.display(), destination.display());
    }

    Ok(())
}

#[entry::gen("bin", "mut", "safe")]
fn entry() -> ! {
    let mut options = CpOptions::new();
    let mut sources = Vec::new();

    argument! {
        args,
        flags: {
            R => options.recursive = true,
            r => options.recursive = true,
            f => options.force = true,
            i => options.interactive = true,
            n => options.no_clobber = true,
            p => options.preserve_attributes = true,
            v => options.verbose = true,
            h => usage!(help->$)
        },
        options: {},
        command: |arg| sources.push(OsStr::from_bytes(arg)),
        on_invalid: |arg| usage!("cp: invalid option -- '{arg}'")
    }

    if sources.len() < 2 {
        usage!();
    }

    let destination = sources.pop().unwrap();
    let destination = Path::new(destination);

    if sources.len() > 1 && !destination.is_dir() {
        error!("cp: target '{}' is not a directory", destination.display());
    }

    for source in sources {
        let source = Path::new(source);
        let dest = if destination.is_dir() {
            destination.join(source.file_name().unwrap())
        } else {
            destination.to_path_buf()
        };

        if let Err(err) = cp(source, &dest, &options) {
            error!("cp: cannot copy '{}' to '{}': {}", source.display(), dest.display(), err);
        }
    }
}
