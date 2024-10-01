#![cfg_attr(feature = "start", feature(start))]

#[cfg(feature = "start")]
#[macro_use]
extern crate macros;

use std::{
    error::Error,
    ffi::{CStr, OsStr},
    fs::{self, OpenOptions},
    os::unix::ffi::OsStrExt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const USAGE: &str = "usage: touch [-c] [-t time] files...";

struct File<'f> {
    path: &'f OsStr,
    no_create: bool,
    time: SystemTime,
}

impl<'f> File<'f> {
    fn new(path: &'f [u8], no_create: bool, time: SystemTime) -> Self {
        Self {
            path: OsStr::from_bytes(path),
            no_create,
            time,
        }
    }

    fn exists(&self) -> bool {
        fs::metadata(&self.path).is_ok()
    }

    fn touch(&self) -> Result<isize, Box<dyn Error>> {
        if self.no_create && !self.exists() {
            error!("touch: cannot touch {:?}: No such file or directory", self.path);
        }

        let file = OpenOptions::new().create(!self.no_create).write(true).open(&self.path)?;

        file.set_modified(self.time)?;

        Ok(0)
    }
}

#[cfg_attr(feature = "start", start)]
pub fn _start(argc: isize, argv: *const *const u8) -> isize {
    let args = (1..argc).map(|arg| unsafe { CStr::from_ptr(*argv.offset(arg) as *const i8).to_bytes() });

    let mut no_create = false;
    let mut time = SystemTime::now();
    let mut files = Vec::new();
    let mut args = args.collect::<Vec<&[u8]>>().into_iter();

    if argc < 2 {
        usage!();
    }

    while let Some(arg) = args.next() {
        match arg {
            b"-c" => no_create = true,
            b"-t" => {
                let time_str = args.next().unwrap_or_else(|| error!("touch: option requires an argument -t"));
                let seconds = std::str::from_utf8(time_str).unwrap_or_else(|_| error!("touch: invalid time: {:?}", time_str)).parse::<u64>();
                time = UNIX_EPOCH + Duration::from_secs(seconds.unwrap_or_else(|_| error!("touch: invalid time: {:?}", time_str)));
            }
            _ => files.push(arg),
        }
    }

    if files.is_empty() {
        usage!();
    }

    for &file in &files {
        let file = File::new(file, no_create, time);

        if let Err(err) = file.touch() {
            error!("touch: {:?}: {}", file.path, err);
        }
    }

    return 0;
}
