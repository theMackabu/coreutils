#![feature(start)]

#[macro_use]
extern crate macros;

use std::{
    error::Error,
    ffi::{CStr, OsStr},
    fs::{self, exists},
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};

const DEFAULT_MODE: u32 = 0o0777;
const USAGE: &str = "usage: mkdir [-p] [-m mode] dir...";

struct Dir<'d> {
    path: &'d OsStr,
    mode: u32,
}

impl<'d> Dir<'d> {
    fn new(path: &'d [u8], mode: u32) -> Self {
        Self { path: OsStr::from_bytes(path), mode }
    }

    fn exists(&self) -> bool {
        match exists(&self.path) {
            Ok(exists) => exists,
            Err(err) => error!("mkdir: {:?} can't create: {err}", self.path),
        }
    }

    fn save(&self) -> Result<isize, Box<dyn Error>> {
        if self.exists() {
            error!("mkdir: {:?} already exists", self.path);
        }

        fs::create_dir(&self.path)?;
        fs::set_permissions(&self.path, fs::Permissions::from_mode(self.mode))?;

        Ok(0)
    }

    fn recursive(&self) -> Result<isize, Box<dyn Error>> {
        let cmp: Vec<&OsStr> = self.path.as_bytes().split(|&b| b == b'/').filter(|s| !s.is_empty()).map(|s| OsStr::from_bytes(s)).collect();

        for i in 0..cmp.len() {
            let path = cmp[..=i].join(OsStr::from_bytes(b"/"));
            let dir = Dir::new(path.as_bytes(), self.mode);

            if i == cmp.len() - 1 && dir.exists() {
                error!("mkdir: {:?} already exists", self.path);
            } else if !dir.exists() {
                dir.save()?;
            }
        }

        Ok(0)
    }
}

#[start]
fn _start(argc: isize, argv: *const *const u8) -> isize {
    let args = (1..argc).map(|arg| unsafe { CStr::from_ptr(*argv.offset(arg) as *const i8).to_bytes() });

    let mut mode = DEFAULT_MODE;
    let mut recursive = false;
    let mut directories = Vec::new();
    let mut args = args.collect::<Vec<&[u8]>>().into_iter();

    if argc < 2 {
        usage!();
    }

    while let Some(arg) = args.next() {
        match arg {
            b"-m" => {
                let bit = args.next().unwrap_or_else(|| error!("mkdir: option requires an argument -m"));
                let mode_str = std::str::from_utf8(bit).unwrap_or_else(|_| error!("mkdir: invalid mode: {bit:?}"));
                mode = u32::from_str_radix(mode_str, 8).unwrap_or_else(|_| error!("mkdir: invalid mode: {mode_str}"));
            }
            b"-p" => recursive = true,
            _ => directories.push(arg),
        }
    }

    for &dir in &directories {
        let dir = Dir::new(dir, mode);

        match recursive {
            true => match dir.recursive() {
                Ok(code) => return code,
                Err(err) => error!("mkdir: {:?}: {err}", dir.path),
            },
            false => match dir.save() {
                Ok(code) => return code,
                Err(err) => error!("mkdir: {:?}: {err}", dir.path),
            },
        }
    }

    return 0;
}
