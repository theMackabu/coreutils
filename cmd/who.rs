#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{mem, ptr};

const USAGE: &str = "usage: who [-aH]";
pub const DESCRIPTION: &str = "Show who is logged on";

#[cfg(target_os = "macos")]
const UTMP_FILE: &str = "/var/run/utmpx";
#[cfg(not(target_os = "macos"))]
const UTMP_FILE: &str = "/var/run/utmp";

#[repr(C)]
struct ExitStatus {
    e_termination: i16,
    e_exit: i16,
}

#[repr(C)]
struct Timeval {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
struct UtmpEntry {
    ut_type: i16,
    ut_pid: i32,
    ut_line: [u8; 32],
    ut_id: [u8; 4],
    ut_user: [u8; 32],
    ut_host: [u8; 256],
    ut_exit: ExitStatus,
    ut_session: i32,
    ut_tv: Timeval,
    ut_addr_v6: [i32; 4],
    __unused: [u8; 20],
}

const USER_PROCESS: i16 = 7;

fn read_utmp() -> io::Result<Vec<UtmpEntry>> {
    let mut file = File::open(UTMP_FILE)?;
    let mut entries = Vec::new();
    let entry_size = mem::size_of::<UtmpEntry>();
    let mut buffer = vec![0u8; entry_size];

    while file.read_exact(&mut buffer).is_ok() {
        let entry = unsafe { ptr::read(buffer.as_ptr() as *const UtmpEntry) };
        entries.push(entry);
    }

    Ok(entries)
}

fn format_time(tv: &Timeval) -> String {
    let time = UNIX_EPOCH + Duration::from_secs(tv.tv_sec as u64);
    let datetime = SystemTime::now().duration_since(time).unwrap_or(Duration::from_secs(0));
    let days = datetime.as_secs() / 86400;
    let hours = (datetime.as_secs() % 86400) / 3600;
    let minutes = (datetime.as_secs() % 3600) / 60;

    if days > 0 {
        format!("{} days", days)
    } else {
        format!("{:02}:{:02}", hours, minutes)
    }
}

fn null_terminated_str(bytes: &[u8]) -> String {
    let nul_range_end = bytes.iter().position(|&c| c == b'\0').unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[0..nul_range_end]).into_owned()
}

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut show_all = false;
    let mut show_headers = false;

    argument! {
        args,
        flags: {
            a => show_all = true,
            H => show_headers = true
        },
        options: {},
        command: |_| usage!(),
        on_invalid: |arg| usage!("who: invalid option -- '{}'", arg as char)
    }

    if show_headers {
        println!("USER\t\tTTY\t\tIDLE\tTIME\t\t HOST");
    }

    match read_utmp() {
        Ok(entries) => {
            for entry in entries {
                if entry.ut_user[0] != 0 && (show_all || entry.ut_type == USER_PROCESS) {
                    let username = null_terminated_str(&entry.ut_user);
                    let tty = null_terminated_str(&entry.ut_line);
                    let host = null_terminated_str(&entry.ut_host);
                    let idle = format_time(&entry.ut_tv);
                    let login_time = UNIX_EPOCH + Duration::from_secs(entry.ut_tv.tv_sec as u64);
                    let login_time_str = format!("{}", login_time.duration_since(UNIX_EPOCH).unwrap().as_secs());

                    println!("{:<15} {:<15} {:<7} {:<16} {}", username, tty, idle, login_time_str, host);
                }
            }
        }
        Err(e) => error!("who: {}", e),
    }
}
