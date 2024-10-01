#![cfg_attr(feature = "start", feature(start))]

#[cfg(feature = "start")]
#[macro_use]
extern crate macros;
extern crate entry;

#[cfg(feature = "start")]
extern crate prelude;

use prelude::*;

const USAGE: &str = "usage: wc [-clmw] [file ...]";

struct WcOptions {
    count_bytes: bool,
    count_chars: bool,
    count_lines: bool,
    count_words: bool,
}

struct CountResult {
    bytes: usize,
    chars: usize,
    lines: usize,
    words: usize,
}

fn count_file(path: &PathBuf, options: &WcOptions) -> Result<CountResult, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    count_reader(&mut reader, options)
}

fn count_stdin(options: &WcOptions) -> Result<CountResult, Box<dyn Error>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    count_reader(&mut reader, options)
}

fn count_reader<R: Read>(reader: &mut R, options: &WcOptions) -> Result<CountResult, Box<dyn Error>> {
    let mut result = CountResult {
        bytes: 0,
        chars: 0,
        lines: 0,
        words: 0,
    };

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    if options.count_bytes {
        result.bytes = buffer.len();
    }

    if options.count_chars {
        result.chars = String::from_utf8_lossy(&buffer).chars().count();
    }

    if options.count_lines {
        result.lines = buffer.iter().filter(|&&c| c == b'\n').count();
    }

    if options.count_words {
        result.words = String::from_utf8_lossy(&buffer).split_whitespace().count();
    }

    Ok(result)
}

fn print_result(result: &CountResult, options: &WcOptions, file_name: Option<&str>) {
    let mut output = String::new();

    if options.count_lines {
        output.push_str(&format!(" {:7} ", result.lines));
    }
    if options.count_words {
        output.push_str(&format!("{:7} ", result.words));
    }
    if options.count_chars {
        output.push_str(&format!("{:7} ", result.chars));
    }
    if options.count_bytes {
        output.push_str(&format!("{:7} ", result.bytes));
    }

    if let Some(name) = file_name {
        output.push_str(name);
    }

    println!("{}", output.trim_end());
}

#[entry::gen(bin)]
fn entry() -> ! {
    let mut files = Vec::new();
    let mut options = WcOptions {
        count_bytes: false,
        count_chars: false,
        count_lines: false,
        count_words: false,
    };

    for arg in args {
        if arg.starts_with(b"-") && arg.len() > 1 {
            for &byte in &arg[1..] {
                match byte {
                    b'c' => options.count_bytes = true,
                    b'm' => options.count_chars = true,
                    b'l' => options.count_lines = true,
                    b'w' => options.count_words = true,
                    _ => {
                        eprintln!("wc: invalid option -- '{}'", byte as char);
                        usage!();
                    }
                }
            }
        } else {
            files.push(PathBuf::from(OsStr::from_bytes(arg)));
        }
    }

    if !(options.count_bytes || options.count_chars || options.count_lines || options.count_words) {
        options.count_bytes = true;
        options.count_lines = true;
        options.count_words = true;
    }

    let mut total = CountResult {
        bytes: 0,
        chars: 0,
        lines: 0,
        words: 0,
    };

    if files.is_empty() {
        match count_stdin(&options) {
            Ok(result) => {
                print_result(&result, &options, None);
            }
            Err(err) => error!("wc: {}", err),
        }
    } else {
        for file in &files {
            match count_file(file, &options) {
                Ok(result) => {
                    print_result(&result, &options, Some(file.to_str().unwrap_or("?")));
                    total.bytes += result.bytes;
                    total.chars += result.chars;
                    total.lines += result.lines;
                    total.words += result.words;
                }
                Err(err) => error!("wc: {}: {}", file.display(), err),
            }
        }

        if files.len() > 1 {
            print_result(&total, &options, Some("total"));
        }
    }

    return 0;
}
