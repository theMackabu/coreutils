// #![cfg_attr(feature = "bin", feature(start))]

// #[cfg(feature = "bin")]
// #[macro_use]
// extern crate macros;
// extern crate entry;

// #[cfg(feature = "bin")]
// extern crate prelude;

// use prelude::*;
// use std::fs::File;
// use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

// const USAGE: &str = "usage: tail [-n lines] [FILE]";

// fn tail_lines<R: Read>(reader: R, num_lines: usize) -> io::Result<Vec<String>> {
//     let mut lines = Vec::new();
//     for line in BufReader::new(reader).lines() {
//         lines.push(line?);
//         if lines.len() > num_lines {
//             lines.remove(0);
//         }
//     }
//     Ok(lines)
// }

// #[entry::gen(cfg = "bin")]
// fn entry() -> ! {
//     let mut num_lines = 10;
//     let mut file_path = None;

//     argument! {
//         args: args,
//         options: {
//             n => {
//                 let lines = args.next().unwrap_or_else(|| usage!("tail: option requires an argument -- 'n'"));
//                 num_lines = lines.parse().unwrap_or_else(|_| usage!("tail: invalid number of lines: '{}'", lines));
//             }
//         },
//         command: |arg| {
//             if file_path.is_some() {
//                 usage!("tail: only one input file may be specified");
//             }
//             file_path = Some(PathBuf::from(OsStr::from_bytes(arg)))
//         },
//         on_invalid: |arg| usage!("tail: invalid option -- '{}'", arg as char)
//     }

//     let result = if let Some(path) = file_path {
//         let file = File::open(path).unwrap_or_else(|e| error!("tail: {}", e));
//         tail_lines(file, num_lines)
//     } else {
//         let stdin = io::stdin();
//         tail_lines(stdin.lock(), num_lines)
//     };

//     match result {
//         Ok(lines) => {
//             for line in lines {
//                 println!("{}", line);
//             }
//         }
//         Err(e) => error!("tail: {}", e),
//     }
// }
