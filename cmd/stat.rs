#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate date;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;
use stat::date::DateTime;
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, UNIX_EPOCH};

const USAGE: &str = "usage: stat [FILE]...";
pub const DESCRIPTION: &str = "Display file or file system status";

fn format_mode(mode: u32) -> String {
    let file_type = match mode & 0o170000 {
        0o040000 => 'd',
        0o020000 => 'c',
        0o060000 => 'b',
        0o010000 => 'p',
        0o140000 => 's',
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

fn format_time(seconds: i64) -> String {
    let time = UNIX_EPOCH + Duration::from_secs(seconds as u64);
    let since = time.duration_since(UNIX_EPOCH).unwrap();
    let dt = DateTime::from_secs(since.as_secs() as i64, false);

    dt.format("%a %b %d %H:%M:%S %Y")
}

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut files = Vec::new();

    argument! {
        args: args,
        options: {},
        command: |arg| files.push(PathBuf::from(OsStr::from_bytes(arg))),
        on_invalid: |arg| usage!("stat: invalid option -- '{}'", arg as char)
    }

    if files.is_empty() {
        files.push(PathBuf::from("."));
    }

    for file in files {
        match fs::metadata(&file) {
            Ok(metadata) => {
                println!("  File: {}", file.display());
                println!("  Size: {:>10}    Blocks: {:>10}    IO Block: {:>10}", metadata.len(), metadata.blocks(), metadata.blksize(),);
                println!("Device: {:>10}    Inode: {:>11}    Links: {:>10}", metadata.dev(), metadata.ino(), metadata.nlink());
                println!("Access: {}    Uid: {:>13}    Gid: {:>13}", format_mode(metadata.mode()), metadata.uid(), metadata.gid());
                println!("Access: {}", format_time(metadata.atime()));
                println!("Modify: {}", format_time(metadata.mtime()));
                println!("Change: {}", format_time(metadata.ctime()));
                println!();
            }
            Err(e) => error!("stat: {}: {}", file.display(), e),
        }
    }
}
