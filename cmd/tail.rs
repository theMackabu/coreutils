#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

use std::time::Duration;

const USAGE: &str = "usage: tail [-f] [-n lines] [FILE]";
pub const DESCRIPTION: &str = "Output or follow the last part of files";

fn tail_lines<R: Read>(reader: R, num_lines: usize) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    for line in BufReader::new(reader).lines() {
        lines.push(line?);
        if lines.len() > num_lines {
            lines.remove(0);
        }
    }
    Ok(lines)
}

fn live_tail<R: Read + BufRead>(mut reader: R, num_lines: usize) -> io::Result<()> {
    let mut lines = tail_lines(&mut reader, num_lines)?;
    for line in lines.iter() {
        println!("{}", line);
    }
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? > 0 {
            if lines.len() >= num_lines {
                lines.remove(0);
            }
            lines.push(line);
            println!("{}", lines.last().unwrap());
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

#[entry::gen("bin", "mut", "safe")]
fn entry() -> ! {
    let mut num_lines = 10;
    let mut file_path = None;
    let mut live = false;

    argument! {
        args.to_owned(),
        name: "tail",
        flags: { f => live = true },
        options: {
            n => |arg| {
                let lines = std::str::from_utf8(arg).unwrap_or_else(|_| usage!("tail: invalid UTF-8 sequence"));
                num_lines = lines.parse().unwrap_or_else(|_| usage!("tail: invalid number of lines: '{lines}'"));
            }
        },
        command: |arg| {
            if file_path.is_some() {
                usage!("tail: only one input file may be specified");
            }
            file_path = Some(PathBuf::from(OsStr::from_bytes(arg)))
        },
        on_invalid: |arg| usage!("tail: invalid option -- '{}'", arg as char)
    }

    if live {
        if let Some(path) = file_path {
            let file = File::open(path).unwrap_or_else(|e| error!("tail: {}", e));
            let reader = BufReader::new(file);
            if let Err(e) = live_tail(reader, num_lines) {
                error!("tail: {}", e);
            }
        } else {
            let stdin = io::stdin();
            if let Err(e) = live_tail(stdin.lock(), num_lines) {
                error!("tail: {}", e);
            }
        }
    } else {
        let result = if let Some(path) = file_path {
            let file = File::open(path).unwrap_or_else(|e| error!("tail: {}", e));
            tail_lines(file, num_lines)
        } else {
            let stdin = io::stdin();
            tail_lines(stdin.lock(), num_lines)
        };
        match result {
            Ok(lines) => {
                for line in lines {
                    println!("{}", line);
                }
            }
            Err(e) => error!("tail: {}", e),
        }
    }
}
