#![feature(rustc_private)]
extern crate libc;

use std::ffi::{CStr, CString};
use std::io;

#[repr(C)]
pub struct Passwd {
    pub pw_name: *mut libc::c_char,
    pub pw_passwd: *mut libc::c_char,
    pub pw_uid: u32,
    pub pw_gid: u32,
    pub pw_gecos: *mut libc::c_char,
    pub pw_dir: *mut libc::c_char,
    pub pw_shell: *mut libc::c_char,
}

#[repr(C)]
pub struct Group {
    pub gr_name: *mut libc::c_char,
    pub gr_passwd: *mut libc::c_char,
    pub gr_gid: u32,
    pub gr_mem: *mut *mut libc::c_char,
}

#[link(name = "c")]
extern "C" {
    pub fn getuid() -> u32;
    pub fn getgrnam(name: *const libc::c_char) -> *mut Group;
    pub fn getpwnam(name: *const libc::c_char) -> *mut Passwd;
    pub fn getpwuid(uid: u32) -> *const Passwd;
    pub fn getgrgid(gid: u32) -> *mut Group;
    pub fn getgroups(size: i32, list: *mut u32) -> i32;
}

pub struct IdOptions {
    pub print_user: bool,
    pub print_group: bool,
    pub print_groups: bool,
    pub use_name: bool,
}

pub fn get_user_info(username: Option<&str>, group: Option<&str>) -> io::Result<(u32, u32, String)> {
    unsafe {
        let passwd = match username {
            Some(name) => {
                let c_name = std::ffi::CString::new(name).unwrap();
                getpwnam(c_name.as_ptr())
            }
            None => getpwuid(getuid()),
        };

        if passwd.is_null() {
            if let Some(name) = group {
                return Ok((0, get_group_id(name)?, String::new()));
            }

            return Err(io::Error::new(io::ErrorKind::NotFound, "User not found"));
        }

        let uid = (*passwd).pw_uid;
        let gid = (*passwd).pw_gid;
        let name = CStr::from_ptr((*passwd).pw_name).to_string_lossy().into_owned();

        Ok((uid, gid, name))
    }
}

pub fn get_group_id(group_name: &str) -> io::Result<u32> {
    unsafe {
        let c_group_name = CString::new(group_name).unwrap();
        let group = getgrnam(c_group_name.as_ptr());
        if group.is_null() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Group not found"));
        }
        Ok((*group).gr_gid)
    }
}

pub fn get_group_name(gid: u32) -> io::Result<String> {
    unsafe {
        let group = getgrgid(gid);
        if group.is_null() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Group not found"));
        }
        Ok(CStr::from_ptr((*group).gr_name).to_string_lossy().into_owned())
    }
}

pub fn get_groups() -> io::Result<Vec<u32>> {
    let mut groups = Vec::with_capacity(32);
    let mut ngroups = groups.capacity() as i32;

    loop {
        unsafe {
            let result = getgroups(ngroups, groups.as_mut_ptr());
            if result >= 0 {
                groups.set_len(result as usize);
                return Ok(groups);
            } else if result == -1 && std::io::Error::last_os_error().raw_os_error() == Some(22) {
                ngroups *= 2;
                groups.reserve(ngroups as usize);
            } else {
                return Err(std::io::Error::last_os_error());
            }
        }
    }
}
