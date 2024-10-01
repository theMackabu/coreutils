#![cfg_attr(feature = "start", feature(start))]

#[cfg(feature = "start")]
#[macro_use]
extern crate macros;

#[cfg(feature = "start")]
extern crate prelude;

use prelude::*;
use std::{
    os::unix::fs::MetadataExt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const USAGE: &str = "usage: ls [-alhrt] [file ...]";

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

fn get_local_offset() -> i64 {
    let local = SystemTime::now();
    let secs = local.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let secs_of_day = secs % 86400;

    if secs_of_day < 43200 {
        (secs_of_day / 3600) as i64
    } else {
        ((secs_of_day - 86400) / 3600) as i64
    }
}

fn format_time(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    let local_offset = get_local_offset();
    let secs = duration.as_secs() as i64 + local_offset * 3600;

    let (year, month, day, hour, minute) = {
        let days = secs / 86400;
        let years = 1970 + (days / 365);
        let days_of_year = days % 365;
        let month = (days_of_year / 30) + 1;
        let day = (days_of_year % 30) + 1;
        let hour = (secs % 86400) / 3600;
        let minute = (secs % 3600) / 60;
        (years, month, day, hour, minute)
    };

    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let month_str = months[(month as usize - 1) % 12];

    format!("{year} {} {:2} {:02}:{:02}", month_str, day, hour, minute)
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

        format!("{:.1}{}", size, units[unit_index])
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
            .map(|entry| entry.path.file_name().unwrap_or(entry.path.as_os_str()).to_string_lossy().len())
            .max()
            .unwrap_or(0);

        let column_width = max_width + 4;
        let terminal_width = std::env::var("COLUMNS").map(|s| s.parse().unwrap_or(80)).unwrap_or(80);
        let columns = std::cmp::min((terminal_width + column_width) / column_width, entries.len());
        let rows = (entries.len() + columns - 1) / columns;

        for row in 0..rows {
            for col in 0..columns {
                if let Some(entry) = entries.get(row + col * rows) {
                    let name = entry.path.file_name().unwrap_or(entry.path.as_os_str()).to_string_lossy();
                    print!("{:<width$}", name, width = column_width);
                }
            }
            println!();
        }
    }

    Ok(())
}

#[cfg_attr(feature = "start", start)]
pub fn _start(argc: isize, argv: *const *const u8) -> isize {
    let args = (1..argc).map(|arg| unsafe { CStr::from_ptr(*argv.offset(arg) as *const i8).to_bytes() });
    let mut options = LsOptions {
        all: false,
        long: false,
        human_readable: false,
        reverse: false,
        sort_by_time: false,
    };
    let mut paths = Vec::new();
    let args = args.collect::<Vec<&[u8]>>();

    for arg in args {
        if arg.starts_with(b"-") && arg.len() > 1 {
            for &byte in &arg[1..] {
                match byte {
                    b'a' => options.all = true,
                    b'l' => options.long = true,
                    b'h' => options.human_readable = true,
                    b'r' => options.reverse = true,
                    b't' => options.sort_by_time = true,
                    _ => {
                        eprintln!("ls: invalid option '{}'", byte as char);
                        usage!();
                    }
                }
            }
        } else {
            paths.push(PathBuf::from(OsStr::from_bytes(arg)));
        }
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

    return 0;
}
