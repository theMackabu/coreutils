#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;
use std::fs::File;
use std::io::{self, BufReader, Read};

const USAGE: &str = "usage: sum [FILE]...";
pub const DESCRIPTION: &str = "Checksum and count the blocks in a file";

fn calculate_sum(mut reader: impl Read) -> io::Result<(u32, u32)> {
    let mut sum = 0u32;
    let mut size = 0u32;
    let mut buffer = [0u8; 4096];

    loop {
        match reader.read(&mut buffer)? {
            0 => break,
            n => {
                sum = sum.wrapping_add(buffer[..n].iter().map(|&b| b as u32).sum::<u32>());
                size += n as u32;
            }
        }
    }

    Ok((sum % 65535, size))
}

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut files = Vec::new();

    argument! {
        args: args,
        options: {},
        command: |arg| files.push(PathBuf::from(OsStr::from_bytes(arg))),
        on_invalid: |arg| usage!("sum: invalid option -- '{}'", arg as char)
    }

    if files.is_empty() {
        let stdin = io::stdin();
        let reader = stdin.lock();
        match calculate_sum(reader) {
            Ok((sum, size)) => println!("{} {}", sum, size),
            Err(e) => error!("sum: {}", e),
        }
    } else {
        for file in files {
            match File::open(&file) {
                Ok(f) => {
                    let reader = BufReader::new(f);
                    match calculate_sum(reader) {
                        Ok((sum, size)) => println!("{} {} {}", sum, size, file.display()),
                        Err(e) => error!("sum: {}: {}", file.display(), e),
                    }
                }
                Err(e) => error!("sum: {}: {}", file.display(), e),
            }
        }
    }
}
