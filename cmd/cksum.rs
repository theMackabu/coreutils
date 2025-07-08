#![cfg_attr(feature = "bin", no_main)]

extern crate entry;
use std::io::{self, Read};

const USAGE: &str = "usage: cksum [FILE...]";
pub const DESCRIPTION: &str = "Calculate the CRC32 checksums of files";

fn crc32_filltable(endian: bool) -> [u32; 256] {
    let polynomial = if endian { 0x04c11db7 } else { 0xedb88320 };
    let mut table = [0u32; 256];

    for i in 0..256 {
        let mut c = if endian { (i as u32) << 24 } else { i as u32 };
        for _ in 0..8 {
            if endian {
                c = if (c & 0x80000000) != 0 { (c << 1) ^ polynomial } else { c << 1 };
            } else {
                c = if (c & 1) != 0 { (c >> 1) ^ polynomial } else { c >> 1 };
            }
        }
        table[i] = c;
    }

    table
}

fn crc32_block_endian1(mut crc: u32, buf: &[u8], crc_table: &[u32; 256]) -> u32 {
    for &byte in buf {
        crc = (crc << 8) ^ crc_table[((crc >> 24) ^ byte as u32) as usize];
    }
    crc
}

fn cksum<R: Read>(mut reader: R) -> io::Result<(u32, u64)> {
    let crc_table = crc32_filltable(true);
    let mut crc = 0u32;
    let mut length = 0u64;
    let mut buffer = [0u8; 4096];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                length += n as u64;
                crc = crc32_block_endian1(crc, &buffer[..n], &crc_table);
            }
            Err(e) => return Err(e),
        }
    }

    let mut len = length;
    while len > 0 {
        crc = (crc << 8) ^ crc_table[((crc >> 24) ^ (len as u8) as u32) as usize];
        len >>= 8;
    }

    Ok((!crc, length))
}

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut files = Vec::new();

    argument! {
        args,
        options: {},
        command: |arg| files.push(PathBuf::from(OsStr::from_bytes(arg))),
        on_invalid: |arg| usage!("cksum: invalid option -- '{}'", arg as char)
    }

    if files.is_empty() {
        match cksum(io::stdin()) {
            Ok((crc, len)) => println!("{} {}", crc, len),
            Err(e) => error!("cksum: {}", e),
        }
    } else {
        for file in files {
            match File::open(&file) {
                Ok(f) => match cksum(f) {
                    Ok((crc, len)) => println!("{} {} {}", crc, len, file.display()),
                    Err(e) => error!("cksum: {}: {}", file.display(), e),
                },
                Err(e) => error!("cksum: {}: {}", file.display(), e),
            }
        }
    }
}
