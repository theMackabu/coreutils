#![cfg_attr(feature = "bin", feature(start))]

extern crate entry;
extern crate env;

use std::ffi::{c_char, c_int, CString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;

const USAGE: &str = "usage: chroot [-g group] [-u user] newroot [command]";
pub const DESCRIPTION: &str = "Run command or interactive shell with special root directory";

type BoxedError = Box<dyn Error>;

fn current_shell() -> String {
    env::get(OsStr::from_bytes(b"SHELL"))
        .unwrap_or_else(|| OsStr::from_bytes(b"/bin/sh").into())
        .to_string_lossy()
        .into_owned()
}

#[link(name = "c")]
extern "C" {
    fn chroot(path: *const c_char) -> c_int;
}

#[entry::gen("bin", "mut")]
fn entry() -> ! {
    let new_root = args
        .next()
        .unwrap_or_else(|| usage!("chroot: missing operand"));
    let new_root = PathBuf::from(OsStr::from_bytes(new_root));

    let command = args.next();
    let command_args: Vec<_> = args.collect();

    let new_root_cstr = CString::new(new_root.as_os_str().as_bytes()).unwrap_or_else(|_| {
        error!("chroot: invalid new root path");
    });

    if chroot(new_root_cstr.as_ptr()) != 0 {
        error!(
            "chroot: failed to change root to {}: {}",
            new_root.display(),
            io::Error::last_os_error()
        );
    }

    if std::env::set_current_dir("/").is_err() {
        error!(
            "chroot: failed to change directory to /: {}",
            io::Error::last_os_error()
        );
    }

    if let Some(cmd) = command {
        let parse = |cmd: &[u8]| -> Result<String, BoxedError> {
            let c_str = CString::new(cmd)?;
            Ok(c_str.into_string().map_err(|e| Box::new(e) as BoxedError)?)
        };

        let cmd = match parse(cmd) {
            Ok(cmd) => cmd,
            Err(err) => error!("chroot: failed to parse command: {err}"),
        };

        let args = match command_args
            .into_iter()
            .map(parse)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(args) => args,
            Err(err) => error!("chroot: failed to parse command arguments: {err}"),
        };

        let err = if args.is_empty() {
            std::process::Command::new(&cmd).exec()
        } else {
            std::process::Command::new(&cmd).args(args).exec()
        };

        error!("chroot: failed to execute {}: {}", cmd, err);
    } else {
        let err = std::process::Command::new(current_shell()).exec();
        eprintln!("chroot: failed to execute shell: {}", err);
    }
}
