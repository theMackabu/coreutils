#![cfg_attr(feature = "bin", no_main)]

extern crate entry;
use std::{fs::exists, os::unix::fs::PermissionsExt};

const DEFAULT_MODE: u32 = 0o0777;
const USAGE: &str = "usage: mkdir [-p] [-m mode] dir...";
pub const DESCRIPTION: &str = "Create directories";

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

#[entry::gen("bin", "mut", "safe")]
fn entry() -> ! {
    let mut mode = DEFAULT_MODE;
    let mut recursive = false;
    let mut directories = Vec::new();

    if argc < 2 {
        usage!();
    }

    argument! {
        args.to_owned(),
        flags: {
            p => recursive = true,
            h => usage!(help->$)
        },
        options: {
            m => |arg: &[u8]| {
                mode = arg.iter().fold(0u32, |acc, &b| {
                    match b {
                        b'0'..=b'7' => acc * 8 + (b - b'0') as u32,
                        _ => error!("mkdir: invalid mode: {arg:?}"),
                    }
                });
            }
        },
        command: |arg| directories.push(arg),
        on_invalid: |arg| usage!("mkdir: invalid option -- '{arg}'")
    }

    for &dir in &directories {
        let dir = Dir::new(dir, mode);
        let result = if recursive { dir.recursive() } else { dir.save() };

        if let Err(err) = result {
            error!("mkdir: {:?}: {err}", dir.path);
        }
    }
}
