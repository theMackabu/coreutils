#![cfg_attr(feature = "bin", no_main)]

extern crate entry;
use std::io::{self, Read, Write};

const USAGE: &str = "usage: base64 <string> [-d] [-i] [-o output_file] [file]";
const BASE64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
pub const DESCRIPTION: &str = "Encode or decode using Base64 representation";

fn encode_base64<R: Read, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    let mut buffer = [0u8; 3];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let b1 = buffer[0] as u32;
                let b2 = if n > 1 { buffer[1] as u32 } else { 0 };
                let b3 = if n > 2 { buffer[2] as u32 } else { 0 };

                let num = (b1 << 16) | (b2 << 8) | b3;

                let e1 = BASE64_ALPHABET[(num >> 18) as usize];
                let e2 = BASE64_ALPHABET[((num >> 12) & 63) as usize];
                let e3 = if n > 1 { BASE64_ALPHABET[((num >> 6) & 63) as usize] } else { b'=' };
                let e4 = if n > 2 { BASE64_ALPHABET[(num & 63) as usize] } else { b'=' };

                writer.write_all(&[e1, e2, e3, e4])?;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn decode_base64<R: Read, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    let mut buffer = [0u8; 4];
    loop {
        match reader.read_exact(&mut buffer) {
            Ok(()) => {
                let mut num = 0u32;
                let mut valid_bytes = 0;
                for &byte in &buffer {
                    num <<= 6;
                    match byte {
                        b'A'..=b'Z' => num |= (byte - b'A') as u32,
                        b'a'..=b'z' => num |= (byte - b'a' + 26) as u32,
                        b'0'..=b'9' => num |= (byte - b'0' + 52) as u32,
                        b'+' => num |= 62,
                        b'/' => num |= 63,
                        b'=' => break,
                        _ => continue,
                    }
                    valid_bytes += 1;
                }

                let b1 = ((num >> 16) & 0xFF) as u8;
                let b2 = ((num >> 8) & 0xFF) as u8;
                let b3 = (num & 0xFF) as u8;

                writer.write_all(&[b1])?;
                if valid_bytes > 2 {
                    writer.write_all(&[b2])?;
                }
                if valid_bytes > 3 {
                    writer.write_all(&[b3])?;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[entry::gen("bin", "mut", "safe")]
fn entry() -> ! {
    let mut decode = false;
    let mut input_file = None;
    let mut output_file = None;
    let mut input_string = None;

    argument! {
        args.to_owned(),
        name: "base64",
        flags: { d => decode = true },
        options: {
            i => |arg| input_file = Some(PathBuf::from(OsStr::from_bytes(arg))),
            o => |arg| output_file = Some(PathBuf::from(OsStr::from_bytes(arg)))
        },
        command: |arg| {
            if input_file.is_some() || input_string.is_some() {
                usage!("base64: too many arguments");
            }
            input_string = Some(OsStr::from_bytes(arg).to_string_lossy().to_string());
        },
        on_invalid: |arg| usage!("base64: invalid option -- '{}'", arg as char)
    }

    let input: Box<dyn Read> = match (input_file, input_string) {
        (Some(path), None) => Box::new(File::open(path).unwrap_or_else(|e| error!("base64: {}", e))),
        (None, Some(string)) => Box::new(io::Cursor::new(string)),
        (None, None) => Box::new(io::stdin()),
        _ => usage!("base64: cannot specify both input file and string"),
    };

    let output: Box<dyn Write> = match output_file {
        Some(ref path) => Box::new(File::create(path).unwrap_or_else(|e| error!("base64: {}", e))),
        None => Box::new(io::stdout()),
    };

    let result = match decode {
        true => decode_base64(input, output),
        false => encode_base64(input, output),
    };

    if let Err(e) = result {
        error!("base64: {}", e);
    }

    if output_file.is_none() {
        println!();
    }

    io::stdout().flush().unwrap_or_else(|e| error!("base64: failed to flush stdout: {}", e));
}
