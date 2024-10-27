#![cfg_attr(feature = "bin", feature(start, rustc_private))]

extern crate entry;

const USAGE: &str = "usage: uname [-asnrvmo]";
pub const DESCRIPTION: &str = "Print system information";

#[cfg(target_os = "macos")]
#[link(name = "System", kind = "framework")]
extern "C" {
    fn sysctl(
        name: *mut i32,
        namelen: u32,
        oldp: *mut std::os::raw::c_void,
        oldlenp: *mut usize,
        newp: *mut std::os::raw::c_void,
        newlen: usize,
    ) -> i32;
    fn sysctlbyname(
        name: *const libc::c_char,
        oldp: *mut std::os::raw::c_void,
        oldlenp: *mut usize,
        newp: *mut std::os::raw::c_void,
        newlen: usize,
    ) -> i32;
}

#[cfg(target_os = "macos")]
const CTL_KERN: i32 = 1;
#[cfg(target_os = "macos")]
const CTL_HW: i32 = 6;
#[cfg(target_os = "macos")]
const KERN_OSTYPE: i32 = 1;
#[cfg(target_os = "macos")]
const KERN_HOSTNAME: i32 = 10;
#[cfg(target_os = "macos")]
const KERN_OSRELEASE: i32 = 2;
#[cfg(target_os = "macos")]
const KERN_VERSION: i32 = 4;
#[cfg(target_os = "macos")]
const HW_MACHINE: i32 = 1;

#[cfg(target_os = "macos")]
fn get_sysctl_string(name: &[i32]) -> Result<String, Box<dyn std::error::Error>> {
    let mut size: usize = 0;
    unsafe {
        if sysctl(
            name.as_ptr() as *mut i32,
            name.len() as u32,
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            return Err("sysctl failed".into());
        }
        let mut buffer = vec![0u8; size];
        if sysctl(
            name.as_ptr() as *mut i32,
            name.len() as u32,
            buffer.as_mut_ptr() as *mut std::os::raw::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            return Err("sysctl failed".into());
        }
        Ok(String::from_utf8(buffer)?
            .trim_end_matches('\0')
            .to_string())
    }
}

#[cfg(target_os = "macos")]
fn get_macos_version() -> Result<String, Box<dyn std::error::Error>> {
    let mut size: usize = 0;
    let name = std::ffi::CString::new("kern.osproductversion")?;
    unsafe {
        if sysctlbyname(
            name.as_ptr(),
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            return Err("sysctlbyname failed".into());
        }
        let mut buffer = vec![0u8; size];
        if sysctlbyname(
            name.as_ptr(),
            buffer.as_mut_ptr() as *mut std::os::raw::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            return Err("sysctlbyname failed".into());
        }
        Ok(String::from_utf8(buffer)?
            .trim_end_matches('\0')
            .to_string())
    }
}

#[cfg(not(target_os = "macos"))]
#[repr(C)]
struct UtsName {
    sysname: [u8; 65],
    nodename: [u8; 65],
    release: [u8; 65],
    version: [u8; 65],
    machine: [u8; 65],
}

#[cfg(not(target_os = "macos"))]
extern "C" {
    fn uname(buf: *mut UtsName) -> i32;
}

struct SysInfo {
    sysname: String,
    nodename: String,
    release: String,
    version: String,
    machine: String,
    #[cfg(target_os = "macos")]
    os_version: String,
}

fn get_sys_info() -> Result<SysInfo, Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        Ok(SysInfo {
            sysname: get_sysctl_string(&[CTL_KERN, KERN_OSTYPE])?,
            nodename: get_sysctl_string(&[CTL_KERN, KERN_HOSTNAME])?,
            release: get_sysctl_string(&[CTL_KERN, KERN_OSRELEASE])?,
            version: get_sysctl_string(&[CTL_KERN, KERN_VERSION])?,
            machine: get_sysctl_string(&[CTL_HW, HW_MACHINE])?,
            os_version: get_macos_version()?,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        use std::mem::MaybeUninit;
        let mut uname_info: MaybeUninit<UtsName> = MaybeUninit::uninit();
        if unsafe { uname(uname_info.as_mut_ptr()) } == 0 {
            let uname_info = unsafe { uname_info.assume_init() };
            Ok(SysInfo {
                sysname: unsafe {
                    CStr::from_ptr(uname_info.sysname.as_ptr() as *const libc::c_char)
                        .to_string_lossy()
                        .into_owned()
                },
                nodename: unsafe {
                    CStr::from_ptr(uname_info.nodename.as_ptr() as *const libc::c_char)
                        .to_string_lossy()
                        .into_owned()
                },
                release: unsafe {
                    CStr::from_ptr(uname_info.release.as_ptr() as *const libc::c_char)
                        .to_string_lossy()
                        .into_owned()
                },
                version: unsafe {
                    CStr::from_ptr(uname_info.version.as_ptr() as *const libc::c_char)
                        .to_string_lossy()
                        .into_owned()
                },
                machine: unsafe {
                    CStr::from_ptr(uname_info.machine.as_ptr() as *const libc::c_char)
                        .to_string_lossy()
                        .into_owned()
                },
            })
        } else {
            Err("uname syscall failed".into())
        }
    }
}

#[entry::gen("bin", "safe", "libc")]
fn entry() -> ! {
    let mut print_sysname = false;
    let mut print_nodename = false;
    let mut print_release = false;
    let mut print_version = false;
    let mut print_machine = false;
    let mut print_os = false;

    argument! {
        args: args,
        options: {
            a => {
                print_sysname = true;
                print_nodename = true;
                print_release = true;
                print_version = true;
                print_machine = true;
                print_os = true;
            },
            s => print_sysname = true,
            n => print_nodename = true,
            r => print_release = true,
            v => print_version = true,
            m => print_machine = true,
            o => print_os = true
        },
        command: |_| usage!(),
        on_invalid: |arg| usage!("uname: invalid option -- '{}'", arg as char)
    }

    if !(print_sysname
        || print_nodename
        || print_release
        || print_version
        || print_machine
        || print_os)
    {
        print_sysname = true;
    }

    match get_sys_info() {
        Ok(info) => {
            let mut output = Vec::new();
            if print_sysname {
                output.push(info.sysname);
            }
            #[cfg(target_os = "macos")]
            if print_os {
                output.push(info.os_version);
            }
            if print_nodename {
                output.push(info.nodename);
            }
            if print_release {
                output.push(info.release);
            }
            if print_version {
                output.push(info.version);
            }
            if print_machine {
                output.push(info.machine);
            }
            println!("{}", output.join(" "));
        }
        Err(e) => error!("uname: {}", e),
    }
}
