#![allow(non_camel_case_types)]
#![cfg_attr(feature = "bin", feature(start, rustc_private))]

extern crate entry;
use std::ffi::CStr;

#[cfg(target_os = "linux")]
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::mem;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;

const USAGE: &str = "usage: df [-k]";
pub const DESCRIPTION: &str = "Report file system disk space usage";

#[cfg(target_os = "macos")]
#[repr(C)]
struct Statfs {
    f_bsize: u32,
    f_iosize: i32,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_fsid: [i32; 2],
    f_owner: libc::uid_t,
    f_type: u32,
    f_flags: u32,
    f_fssubtype: u32,
    f_fstypename: [libc::c_char; 16],
    f_mntonname: [libc::c_char; 1024],
    f_mntfromname: [libc::c_char; 1024],
    f_reserved: [u32; 8],
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct Statfs {
    f_type: libc::c_long,
    f_bsize: libc::c_long,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_fsid: [libc::c_int; 2],
    f_namelen: libc::c_long,
    f_frsize: libc::c_long,
    f_flags: libc::c_long,
    f_spare: [libc::c_long; 4],
}

#[cfg(target_os = "macos")]
extern "C" {
    fn getmntinfo(mntbufp: *mut *mut Statfs, flags: libc::c_int) -> libc::c_int;
}

#[cfg(target_os = "linux")]
extern "C" {
    fn setmntent(filename: *const libc::c_char, type_: *const libc::c_char) -> *mut libc::FILE;
    fn getmntent(stream: *mut libc::FILE) -> *mut libc::mntent;
    fn endmntent(stream: *mut libc::FILE) -> libc::c_int;
    fn statfs(path: *const libc::c_char, buf: *mut Statfs) -> libc::c_int;
}

fn format_size(size: u64, use_512_blocks: bool) -> String {
    if use_512_blocks {
        (size * 512 / 1024).to_string()
    } else {
        size.to_string()
    }
}

#[cfg(target_os = "macos")]
fn print_df(statfs_buf: &Statfs, use_512_blocks: bool) -> io::Result<()> {
    let block_size = if use_512_blocks { 512 } else { statfs_buf.f_bsize as u64 };

    let total = statfs_buf.f_blocks * block_size / 512;
    let available = statfs_buf.f_bavail * block_size / 512;
    let used = total - (statfs_buf.f_bfree * block_size / 512);
    let capacity = if total > 0 { (used as f64 / total as f64 * 100.0) as u64 } else { 0 };

    let filesystem = unsafe { CStr::from_ptr(statfs_buf.f_mntfromname.as_ptr()).to_string_lossy() };
    let mount_point = unsafe { CStr::from_ptr(statfs_buf.f_mntonname.as_ptr()).to_string_lossy() };

    let iused = statfs_buf.f_files - statfs_buf.f_ffree;
    let ifree = statfs_buf.f_ffree;
    let iused_percent = if statfs_buf.f_files > 0 { (iused as f64 / statfs_buf.f_files as f64 * 100.0) as u64 } else { 0 };

    println!(
        "{:<15} {:>10} {:>10} {:>10} {:>3}% {:>7} {:>9} {:>5}%  {}",
        filesystem,
        format_size(total, use_512_blocks),
        format_size(used, use_512_blocks),
        format_size(available, use_512_blocks),
        capacity,
        iused,
        ifree,
        iused_percent,
        mount_point
    );

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_df(mntent: &libc::mntent, use_512_blocks: bool) -> io::Result<()> {
    let mut statfs_buf: Statfs = unsafe { mem::zeroed() };
    let c_path = CString::new(mntent.mnt_dir)?;

    if unsafe { statfs(c_path.as_ptr(), &mut statfs_buf) } == -1 {
        return Err(io::Error::last_os_error());
    }

    let block_size = if use_512_blocks { 512 } else { statfs_buf.f_bsize as u64 };

    let total = statfs_buf.f_blocks * block_size / 512;
    let available = statfs_buf.f_bavail * block_size / 512;
    let used = total - (statfs_buf.f_bfree * block_size / 512);
    let capacity = if total > 0 { (used as f64 / total as f64 * 100.0) as u64 } else { 0 };

    let filesystem = unsafe { CStr::from_ptr(mntent.mnt_fsname).to_string_lossy() };
    let mount_point = unsafe { CStr::from_ptr(mntent.mnt_dir).to_string_lossy() };

    let iused = statfs_buf.f_files - statfs_buf.f_ffree;
    let ifree = statfs_buf.f_ffree;
    let iused_percent = if statfs_buf.f_files > 0 { (iused as f64 / statfs_buf.f_files as f64 * 100.0) as u64 } else { 0 };

    println!(
        "{:<15} {:>10} {:>10} {:>10} {:>3}% {:>7} {:>9} {:>5}%  {}",
        filesystem,
        format_size(total, use_512_blocks),
        format_size(used, use_512_blocks),
        format_size(available, use_512_blocks),
        capacity,
        iused,
        ifree,
        iused_percent,
        mount_point
    );

    Ok(())
}

#[entry::gen("bin", "libc")]
fn entry() -> ! {
    let mut use_512_blocks = true;

    argument! {
        args: args,
        options: {
            k => use_512_blocks = false
        },
        command: |_| {},
        on_invalid: |arg| usage!("df: invalid option -- '{}'", arg as char)
    }

    println!(
        "{:<15} {:>10} {:>10} {:>10} {:>3} {:>7} {:>9} {:>5}  {}",
        "Filesystem", "512-blocks", "Used", "Available", "Capacity", "iused", "ifree", "%iused", "Mounted on"
    );

    #[cfg(target_os = "macos")]
    {
        let mut fs_info: *mut Statfs = std::ptr::null_mut();
        let fs_count = getmntinfo(&mut fs_info, 0);

        if fs_count <= 0 {
            error!("df: Failed to get filesystem information");
        }

        for i in 0..fs_count {
            let statfs_buf = &*fs_info.offset(i as isize);
            if let Err(e) = print_df(statfs_buf, use_512_blocks) {
                error!("df: Failed to print filesystem information: {}", e);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let mtab = CString::new("/etc/mtab").unwrap();
        let mode = CString::new("r").unwrap();
        let file = setmntent(mtab.as_ptr(), mode.as_ptr());

        if file.is_null() {
            error!("df: Failed to open /etc/mtab");
        }

        loop {
            let ent = getmntent(file);
            if ent.is_null() {
                break;
            }
            let mntent = &*ent;
            if let Err(e) = print_df(mntent, use_512_blocks) {
                error!("df: Failed to print filesystem information: {}", e);
            }
        }

        unsafe { endmntent(file) };
    }
}
