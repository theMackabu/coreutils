#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;
use std::fmt;
use std::os::unix::fs::PermissionsExt;

const USAGE: &str = "\
usage:  chmod [-fhv] [-R [-H | -L | -P]] [-a | +a | =a  [i][# [ n]]] mode|entry file ...
        chmod [-fhv] [-R [-H | -L | -P]] [-E | -C | -N | -i] file ...";

pub const COMMAND: (&str, &str) = ("chmod", "Change file mode bits");

struct ChmodOptions {
    force: bool,
    verbose: bool,
    recursive: bool,
    dereference: bool,
    no_dereference: bool,
    preserve_root: bool,
    acl_manipulation: Option<String>,
    extended_acl: bool,
    clear_acl: bool,
    remove_acl: bool,
    inherit_acl: bool,
}

#[derive(Debug)]
struct ChmodError(String);

impl fmt::Display for ChmodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ChmodError {}

fn parse_mode(mode: &str, current_mode: u32) -> Result<u32, ChmodError> {
    if mode.starts_with('+') || mode.starts_with('-') || mode.starts_with('=') {
        let mut new_mode = current_mode;
        let (op, rest) = mode.split_at(1);
        for c in rest.chars() {
            let bits = match c {
                'r' => 0o444,
                'w' => 0o222,
                'x' => 0o111,
                'X' => {
                    if current_mode & 0o111 != 0 {
                        0o111
                    } else {
                        0
                    }
                }
                's' => 0o4000 | 0o2000,
                't' => 0o1000,
                _ => return Err(ChmodError(format!("Invalid mode character: {}", c))),
            };
            match op {
                "+" => new_mode |= bits,
                "-" => new_mode &= !bits,
                "=" => new_mode = (new_mode & 0o7000) | bits,
                _ => unreachable!(),
            }
        }
        Ok(new_mode)
    } else if mode.chars().all(|c| c.is_digit(8)) {
        u32::from_str_radix(mode, 8).map_err(|_| ChmodError("Invalid octal mode".to_string()))
    } else {
        Err(ChmodError("Invalid mode format".to_string()))
    }
}

fn chmod_recursive(path: &Path, mode: &str, options: &ChmodOptions) -> io::Result<()> {
    let metadata = if options.no_dereference { fs::symlink_metadata(path)? } else { fs::metadata(path)? };

    let current_mode = metadata.permissions().mode();
    let new_mode = parse_mode(mode, current_mode).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.0))?;

    fs::set_permissions(path, fs::Permissions::from_mode(new_mode))?;

    if options.verbose {
        println!("changed '{}' mode from {:o} to {:o}", path.display(), current_mode, new_mode);
    }

    if metadata.is_dir() && options.recursive {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            chmod_recursive(&entry.path(), mode, options)?;
        }
    }

    Ok(())
}

#[entry::gen(cfg = "bin")]
fn entry() -> ! {
    let mut options = ChmodOptions {
        force: false,
        verbose: false,
        recursive: false,
        dereference: false,
        no_dereference: false,
        preserve_root: true,
        acl_manipulation: None,
        extended_acl: false,
        clear_acl: false,
        remove_acl: false,
        inherit_acl: false,
    };

    let mut mode = None;
    let mut files = Vec::new();

    argument! {
        args: args,
        options: {
            f => options.force = true,
            h => options.no_dereference = true,
            v => options.verbose = true,
            R => options.recursive = true,
            H => options.dereference = true,
            L => options.dereference = true,
            P => options.preserve_root = true,
            E => options.extended_acl = true,
            C => options.clear_acl = true,
            N => options.remove_acl = true,
            i => options.inherit_acl = true,
        },
        command: |arg| {
            let arg = String::from_utf8_lossy(arg).into_owned();
            if mode.is_none() {
                mode = Some(arg);
            } else {
                files.push(PathBuf::from(arg));
            }
        },
        on_invalid: |arg| usage!("chmod: invalid option -- '{}'", arg as char)
    }

    if files.is_empty() {
        usage!();
    }

    let mode = mode.unwrap_or_else(|| usage!("chmod: missing mode operand"));

    for file in files {
        let result = chmod_recursive(&file, &mode, &options);

        if let Err(e) = result {
            if options.force {
                eprintln!("chmod: changing permissions of '{}': {}", file.display(), e);
            } else {
                error!("chmod: changing permissions of '{}': {}", file.display(), e);
            }
        }
    }
}
