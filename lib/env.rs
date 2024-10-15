#![feature(rustc_private)]
extern crate libc;

#[path = "macros.rs"]
mod macros;

use std::{
    ffi::{CStr, CString, OsStr, OsString},
    io,
    mem::MaybeUninit,
    os::{unix::ffi::OsStrExt, unix::prelude::OsStringExt},
    path::Path,
    ptr, slice,
    vec::IntoIter,
};

#[cfg(not(target_os = "espidf"))]
const MAX_STACK_ALLOCATION: usize = 384;
#[cfg(target_os = "espidf")]
const MAX_STACK_ALLOCATION: usize = 32;

pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

#[link(name = "c")]
extern "C" {
    fn getenv(s: *const libc::c_char) -> *mut libc::c_char;
}

#[cfg(target_os = "macos")]
#[link(name = "c")]
extern "C" {
    fn _NSGetEnviron() -> *mut *mut *mut libc::c_char;
}

pub struct Vars {
    inner: IntoIter<OsString>,
}

impl Iterator for Vars {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.inner.next().map(|var| var.into_string().unwrap())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[cfg(target_os = "macos")]
unsafe fn environ() -> *mut *const *const libc::c_char {
    _NSGetEnviron() as *mut *const *const libc::c_char
}

#[cfg(not(target_os = "macos"))]
unsafe fn environ() -> *mut *const *const libc::c_char {
    extern "C" {
        static mut environ: *const *const libc::c_char;
    }
    std::ptr::addr_of_mut!(environ)
}

unsafe fn run_with_cstr_stack<T>(bytes: &[u8], f: &dyn Fn(&CStr) -> io::Result<T>) -> io::Result<T> {
    let mut buf = MaybeUninit::<[u8; MAX_STACK_ALLOCATION]>::uninit();
    let buf_ptr = buf.as_mut_ptr() as *mut u8;

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf_ptr, bytes.len());
        buf_ptr.add(bytes.len()).write(0);
    }

    match CStr::from_bytes_with_nul(unsafe { slice::from_raw_parts(buf_ptr, bytes.len() + 1) }) {
        Ok(s) => f(s),
        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidInput, "file name contained an unexpected NUL byte")),
    }
}

#[cold]
#[inline(never)]
fn run_with_cstr_allocating<T>(bytes: &[u8], f: &dyn Fn(&CStr) -> io::Result<T>) -> io::Result<T> {
    match CString::new(bytes) {
        Ok(s) => f(&s),
        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidInput, "file name contained an unexpected NUL byte")),
    }
}

#[inline]
pub fn run_path_with_cstr<T>(path: &Path, f: &dyn Fn(&CStr) -> io::Result<T>) -> io::Result<T> {
    run_with_cstr(path.as_os_str().as_encoded_bytes(), f)
}

#[inline]
pub fn run_with_cstr<T>(bytes: &[u8], f: &dyn Fn(&CStr) -> io::Result<T>) -> io::Result<T> {
    if bytes.len() >= MAX_STACK_ALLOCATION {
        run_with_cstr_allocating(bytes, f)
    } else {
        unsafe { run_with_cstr_stack(bytes, f) }
    }
}

pub fn vars() -> Vec<String> {
    unsafe {
        let mut environ = *environ();
        let mut result = Vec::new();

        if !environ.is_null() {
            while !(*environ).is_null() {
                if let Some(key_value) = parse(CStr::from_ptr(*environ).to_bytes()) {
                    result.push(key_value);
                }
                environ = environ.add(1);
            }
        }

        return Vars { inner: result.into_iter() }.collect();
    }

    fn parse(input: &[u8]) -> Option<OsString> {
        if input.is_empty() {
            return None;
        }
        Some(OsString::from_vec(input.to_vec()))
    }
}

pub fn get(k: &OsStr) -> Option<OsString> {
    run_with_cstr(k.as_bytes(), &|k| {
        let v = unsafe { getenv(k.as_ptr()) } as *const libc::c_char;

        if v.is_null() {
            Ok(None)
        } else {
            let bytes = unsafe { CStr::from_ptr(v) }.to_bytes().to_vec();

            Ok(Some(OsStringExt::from_vec(bytes)))
        }
    })
    .ok()
    .flatten()
}

pub fn cvt<T: IsMinusOne>(t: T) -> crate::io::Result<T> {
    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}
