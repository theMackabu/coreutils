#![cfg_attr(feature = "bin", feature(start))]

extern crate entry;

use std::process::Command;
use std::time::Instant;

const RUSAGE_CHILDREN: i32 = -1;
const USAGE: &str = "usage: time COMMAND [ARGS]";
pub const DESCRIPTION: &str = "Run programs and summarize system resource usage";

#[repr(C)]
#[derive(Clone)]
struct Timeval {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
struct Rusage {
    ru_utime: Timeval,
    ru_stime: Timeval,
}

#[link(name = "c")]
extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
}

fn get_resource_usage() -> Rusage {
    let time = Timeval { tv_sec: 0, tv_usec: 0 };

    let mut usage = Rusage {
        ru_utime: time.to_owned(),
        ru_stime: time.to_owned(),
    };

    unsafe {
        getrusage(RUSAGE_CHILDREN, &mut usage);
    }
    return usage;
}

fn timeval_to_seconds(tv: &Timeval) -> f64 {
    tv.tv_sec as f64 + tv.tv_usec as f64 * 1e-6
}

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let args = args.map(|a| String::from_utf8_lossy(a).to_string()).collect::<Vec<_>>();

    if args.is_empty() {
        usage!("time: missing command");
    }

    let start = Instant::now();
    let status = Command::new(&args[0]).args(&args[1..]).status().unwrap();

    let duration = start.elapsed();
    let usage = get_resource_usage();

    print_time_info(&args[0], usage, duration);

    if let Some(code) = status.code() {
        if code != 0 {
            std::process::exit(code);
        }
    }
}

fn print_time_info(program: &str, usage: Rusage, real_time: std::time::Duration) {
    let user_time = timeval_to_seconds(&usage.ru_utime);
    let system_time = timeval_to_seconds(&usage.ru_stime);
    let real_time_secs = real_time.as_secs_f64();
    let cpu_usage = ((user_time + system_time) / real_time_secs * 100.0) as u32;

    println!("{}  {:.2}s user {:.2}s system {}% cpu {:.3} total", program, user_time, system_time, cpu_usage, real_time_secs);
}
