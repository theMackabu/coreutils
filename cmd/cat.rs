#![feature(start)]
#[macro_use]
extern crate macros;

use std::{
    error::Error,
    ffi::{CStr, OsStr},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    os::unix::ffi::OsStrExt,
    path::Path,
};

const USAGE: &str = "usage: cat [-benstv] [file ...]";

struct CatOptions {
    number_nonblank: bool,
    show_ends: bool,
    number: bool,
    squeeze_blank: bool,
    show_tabs: bool,
    show_nonprinting: bool,
}

fn cat_file<R: BufRead>(reader: R, options: &CatOptions) -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut line_number = 1;
    let mut last_line_blank = false;

    for line in reader.lines() {
        let mut line = line?;
        let is_blank = line.trim().is_empty();

        if options.squeeze_blank && is_blank && last_line_blank {
            continue;
        }

        if options.show_nonprinting {
            line = line.chars().map(|c| if c.is_ascii_control() && c != '\t' && c != '\n' { (c as u8 + 64) as char } else { c }).collect();
        }

        if options.show_tabs {
            line = line.replace('\t', "^I");
        }

        if (options.number && !options.number_nonblank) || (options.number_nonblank && !is_blank) {
            write!(stdout, "{:6}\t", line_number)?;
            line_number += 1;
        }

        stdout.write_all(line.as_bytes())?;

        if options.show_ends {
            stdout.write_all(b"$")?;
        }

        stdout.write_all(b"\n")?;

        last_line_blank = is_blank;
    }

    Ok(())
}

#[start]
fn main(argc: isize, argv: *const *const u8) -> isize {
    let args = (1..argc).map(|arg| unsafe { CStr::from_ptr(*argv.offset(arg) as *const i8).to_bytes() });
    let mut options = CatOptions {
        number_nonblank: false,
        show_ends: false,
        number: false,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: false,
    };
    let mut files = Vec::new();
    let mut args = args.collect::<Vec<&[u8]>>().into_iter();

    if argc < 2 {
        usage!();
    }

    while let Some(arg) = args.next() {
        match arg {
            b"-b" => options.number_nonblank = true,
            b"-e" => {
                options.show_ends = true;
                options.show_nonprinting = true;
            }
            b"-n" => options.number = true,
            b"-s" => options.squeeze_blank = true,
            b"-t" => {
                options.show_tabs = true;
                options.show_nonprinting = true;
            }
            b"-v" => options.show_nonprinting = true,
            _ => files.push(OsStr::from_bytes(arg)),
        }
    }

    if files.is_empty() {
        let stdin = io::stdin();
        let reader = stdin.lock();
        if let Err(err) = cat_file(reader, &options) {
            error!("cat: <stdin>: {}", err);
        }
    } else {
        for file in files {
            let path = Path::new(file);
            let file = match File::open(path) {
                Ok(file) => file,
                Err(err) => error!("cat: {}: {}", path.display(), err),
            };
            let reader = BufReader::new(file);
            if let Err(err) = cat_file(reader, &options) {
                error!("cat: {}: {}", path.display(), err);
            }
        }
    }

    0
}
