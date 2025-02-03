#![cfg_attr(feature = "bin", no_main, feature(rustc_private))]

extern crate entry;

const USAGE: &str = "usage: kill [-s SIGNAL] PID...";
pub const DESCRIPTION: &str = "Send a signal to a process";

#[link(name = "c")]
extern "C" {
    fn kill(pid: libc::pid_t, sig: libc::c_int) -> libc::c_int;
}

#[entry::gen("bin", "mut", "libc")]
fn entry() -> ! {
    let mut signal = libc::SIGTERM;
    let mut pids = Vec::new();

    argument! {
        args: args.to_owned(),
        options: {
            s => {
                let sig = args.next().unwrap_or_else(|| usage!("kill: option requires an argument -- 's'"));
                signal = match String::from_utf8_lossy(sig).parse() {
                    Ok(sig) => sig,
                    Err(_) => usage!("kill: invalid signal")
                };
            }
        },
        command: |arg| {
            pids.push(String::from_utf8_lossy(arg).parse::<i32>().unwrap_or_else(|_| usage!("kill: invalid PID")));
        },
        on_invalid: |arg| usage!("kill: invalid option -- '{}'", arg as char)
    }

    if pids.is_empty() {
        usage!("kill: PID argument required");
    }

    for pid in pids {
        if kill(pid, signal) == -1 {
            error!("kill: failed to send signal to process {}: {}", pid, io::Error::last_os_error());
        }
    }
}
