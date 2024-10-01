pub use std::{
    error::Error,
    ffi::{CStr, OsStr},
    fs::{self, File, Metadata, OpenOptions},
    io::{self, BufRead, BufReader, Read, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

pub fn parse_args(argc: isize, argv: *const *const u8) -> Vec<&'static [u8]> {
    (1..argc).map(|arg| unsafe { CStr::from_ptr(*argv.offset(arg) as *const i8).to_bytes() }).collect::<Vec<&[u8]>>()
}
