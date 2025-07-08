#![cfg_attr(feature = "bin", no_main)]

extern crate entry;
use std::os::unix::fs::MetadataExt;

const USAGE: &str = "usage: du [-ahsH] [file ...]";
pub const DESCRIPTION: &str = "Estimate file space usage";

struct DuOptions {
    all: bool,
    human_readable: bool,
    summarize: bool,
    dereference: bool,
}

fn format_size(size: u64, human_readable: bool) -> String {
    if human_readable {
        let units = ["", "K", "M", "G", "T", "P"];
        let mut size = size as f64 * 512.0;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < units.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        match unit_index {
            0 => format!("{:>4}", size as u64),
            1 => format!("{:>3.0}K", size),
            _ => format!("{:>3.1}{}", size, units[unit_index]),
        }
    } else {
        size.to_string()
    }
}

fn du(path: &Path, options: &DuOptions) -> Result<u64, Box<dyn Error>> {
    let metadata = if options.dereference { fs::metadata(path)? } else { fs::symlink_metadata(path)? };
    let mut total_size = metadata.blocks();

    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let size = du(&entry.path(), options)?;
            total_size += size;
        }

        if !options.summarize || path == Path::new(".") {
            println!("{}\t{}", format_size(total_size, options.human_readable), path.display());
        }
    } else if options.all {
        println!("{}\t{}", format_size(total_size, options.human_readable), path.display());
    }

    Ok(total_size)
}

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut paths = Vec::new();

    let mut options = DuOptions {
        all: false,
        human_readable: false,
        summarize: false,
        dereference: false,
    };

    argument! {
        args,
        flags: {
            a => options.all = true,
            h => options.human_readable = true,
            s => options.summarize = true,
            H => options.dereference = true
        },
        options: {},
        command: |arg| paths.push(PathBuf::from(OsStr::from_bytes(arg))),
        on_invalid: |arg| usage!("du: invalid option -- '{arg}'")
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    let mut total_size = 0;

    for path in &paths {
        match du(path, &options) {
            Ok(size) => {
                total_size += size;
                if options.summarize && path != Path::new(".") {
                    println!("{}\t{}", size, path.display());
                }
            }
            Err(err) => error!("du: {}: {}", path.display(), err),
        }
    }

    if options.summarize && paths.len() > 1 {
        println!("{}\ttotal", total_size);
    }
}
