#![cfg_attr(feature = "bin", feature(start))]

extern crate date;
extern crate entry;

use self::date::DateTime;
use std::{
    os::unix::fs::MetadataExt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const USAGE: &str = "usage: ls [-alhrt] [file ...]";
pub const DESCRIPTION: &str = "List directory contents";

struct LsOptions {
    all: bool,
    long: bool,
    human_readable: bool,
    reverse: bool,
    sort_by_time: bool,
}

struct FileInfo {
    path: PathBuf,
    metadata: Metadata,
}

fn format_mode(mode: u32) -> String {
    let file_type = match mode & 0o170000 {
        0o040000 => 'd',
        0o120000 => 'l',
        _ => '-',
    };

    let permissions = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];

    let mut result = String::with_capacity(10);
    result.push(file_type);
    for &(mask, ch) in &permissions {
        result.push(if mode & mask != 0 { ch } else { '-' });
    }
    result
}

fn format_time(time: SystemTime) -> String {
    let since = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    let dt = DateTime::from_secs(since.as_secs() as i64, false);

    dt.format("%Y %b %r %H:%M")
}

fn format_size(size: u64, human_readable: bool) -> String {
    if human_readable {
        let units = ["B", "K", "M", "G", "T", "P"];
        let mut size = size as f64;
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

fn list_directory(path: &Path, options: &LsOptions) -> Result<Vec<FileInfo>, Box<dyn Error>> {
    let mut entries = fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let file_name = entry.file_name();
            let name = file_name.to_str().unwrap_or("");
            options.all || (!name.starts_with('.') && name != "." && name != "..")
        })
        .map(|entry| {
            let path = entry.path();
            let metadata = entry.metadata()?;
            Ok(FileInfo { path, metadata })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    if options.all {
        if let Ok(metadata) = fs::metadata(path) {
            entries.insert(0, FileInfo { path: PathBuf::from("."), metadata });
        }

        if let Some(parent_path) = path.parent() {
            if let Ok(metadata) = fs::metadata(parent_path) {
                entries.insert(1, FileInfo { path: PathBuf::from(".."), metadata });
            }
        } else {
            if let Ok(metadata) = fs::metadata(path) {
                entries.insert(1, FileInfo { path: PathBuf::from(".."), metadata });
            }
        }
    }

    entries.sort_by(|a, b| {
        if options.sort_by_time {
            let order = b.metadata.modified().unwrap().cmp(&a.metadata.modified().unwrap());
            if options.reverse {
                order.reverse()
            } else {
                order
            }
        } else {
            let order = a.path.file_name().cmp(&b.path.file_name());
            if options.reverse {
                order.reverse()
            } else {
                order
            }
        }
    });

    Ok(entries)
}

fn display_entries(entries: &[FileInfo], options: &LsOptions) -> Result<(), Box<dyn Error>> {
    if options.long {
        let total_blocks: u64 = entries.iter().map(|e| e.metadata.blocks()).sum();
        println!("total {}", total_blocks);

        for entry in entries {
            let mode = format_mode(entry.metadata.mode());
            let nlink = entry.metadata.nlink();
            let uid = entry.metadata.uid();
            let gid = entry.metadata.gid();
            let size = format_size(entry.metadata.size(), options.human_readable);
            let time = format_time(entry.metadata.modified()?);
            let name = entry.path.file_name().unwrap_or(entry.path.as_os_str()).to_string_lossy();

            println!("{} {:>3} {:>5} {:>5} {:>6} {} {}", mode, nlink, uid, gid, size, time, name);
        }
    } else {
        let max_width = entries
            .iter()
            .map(|entry| entry.path.file_name().unwrap_or(entry.path.as_os_str()).to_str().unwrap_or("").len())
            .max()
            .unwrap_or(0);

        let column_width = max_width + 3;
        let terminal_width = std::env::var("COLUMNS").map(|s| s.parse().unwrap_or(80)).unwrap_or(80);

        let columns = std::cmp::min((terminal_width + column_width) / column_width, entries.len());
        let rows = (entries.len() + columns - 1) / columns;

        let mut grid: Vec<Vec<&str>> = vec![vec![]; rows];

        for (index, entry) in entries.iter().enumerate() {
            let row = index % rows;
            let col = index / rows;
            let name = entry.path.file_name().unwrap_or(entry.path.as_os_str());

            if col >= grid[row].len() {
                grid[row].resize(col + 1, "");
            }

            grid[row][col] = name.to_str().unwrap_or("");
        }

        for row in 0..rows {
            for col in 0..columns {
                if let Some(name) = grid[row].get(col) {
                    print!("{:<width$}", name, width = column_width);
                }
            }
            println!();
        }
    }

    Ok(())
}

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut paths = Vec::new();

    let mut options = LsOptions {
        all: false,
        long: false,
        human_readable: false,
        reverse: false,
        sort_by_time: false,
    };

    argument! {
        args: args,
        options: {
            a => options.all = true,
            l => options.long = true,
            h => options.human_readable = true,
            r => options.reverse = true,
            t => options.sort_by_time = true
        },
        command: |arg| paths.push(PathBuf::from(OsStr::from_bytes(arg))),
        on_invalid: |arg| usage!("ls: invalid option -- '{arg}'")
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    let multiple_paths = paths.len() > 1;

    for path in &paths {
        if multiple_paths {
            println!("{}:", path.display());
        }

        match list_directory(path, &options) {
            Ok(entries) => {
                if let Err(err) = display_entries(&entries, &options) {
                    error!("ls: {}: {}", path.display(), err);
                }
            }
            Err(err) => error!("ls: {}: {}", path.display(), err),
        }

        if multiple_paths {
            println!();
        }
    }
}
