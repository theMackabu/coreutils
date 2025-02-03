#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

use std::io::{self, Read};

const SHA1DLEN: usize = 20;
const USAGE: &str = "usage: sha1sum [file...]";
pub const DESCRIPTION: &str = "Compute and check SHA1 message digest";

struct DigestState {
    len: u64,
    state: [u32; 5],
    buf: [u8; 64],
    blen: usize,
}

impl DigestState {
    fn new() -> Self {
        DigestState {
            len: 0,
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
            buf: [0; 64],
            blen: 0,
        }
    }
}

fn sha1block(state: &mut [u32; 5], buf: &[u8]) {
    let mut w = [0u32; 80];
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];

    for i in 0..16 {
        w[i] = (buf[i * 4] as u32) << 24 | (buf[i * 4 + 1] as u32) << 16 | (buf[i * 4 + 2] as u32) << 8 | (buf[i * 4 + 3] as u32);
    }

    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }

    for i in 0..80 {
        let f = match i {
            0..=19 => (b & c) | (!b & d),
            20..=39 => b ^ c ^ d,
            40..=59 => (b & c) | (b & d) | (c & d),
            _ => b ^ c ^ d,
        };
        let k = match i {
            0..=19 => 0x5a827999,
            20..=39 => 0x6ed9eba1,
            40..=59 => 0x8f1bbcdc,
            _ => 0xca62c1d6,
        };
        let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = temp;
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
}

fn sha1(buf: &[u8], mut s: DigestState) -> (DigestState, [u8; SHA1DLEN]) {
    let mut state = s.state;
    let mut len = s.len;
    let mut p = buf;

    if s.blen > 0 {
        let n = std::cmp::min(64 - s.blen, p.len());
        s.buf[s.blen..s.blen + n].copy_from_slice(&p[..n]);
        s.blen += n;
        p = &p[n..];
        len += n as u64;

        if s.blen == 64 {
            sha1block(&mut state, &s.buf);
            s.blen = 0;
        }
    }

    while p.len() >= 64 {
        sha1block(&mut state, &p[..64]);
        p = &p[64..];
        len += 64;
    }

    if !p.is_empty() {
        s.buf[..p.len()].copy_from_slice(p);
        s.blen = p.len();
        len += p.len() as u64;
    }

    s.len = len;
    s.state = state;

    let mut digest = [0u8; SHA1DLEN];
    for i in 0..5 {
        digest[i * 4] = (state[i] >> 24) as u8;
        digest[i * 4 + 1] = (state[i] >> 16) as u8;
        digest[i * 4 + 2] = (state[i] >> 8) as u8;
        digest[i * 4 + 3] = state[i] as u8;
    }

    (s, digest)
}

fn sha1_final(s: DigestState) -> [u8; SHA1DLEN] {
    let len = s.len;
    let mut buf = [0u8; 128];

    buf[0] = 0x80;
    let mut n: usize = 1 + ((119 - (len % 64)) % 64) as usize;
    buf[n..n + 8].copy_from_slice(&(len * 8).to_be_bytes());
    n += 8;

    let (_, digest) = sha1(&buf[..n], s);
    digest
}

fn sum(mut reader: impl Read, name: Option<&str>) -> io::Result<()> {
    let mut buf = [0u8; 8192];
    let mut s = DigestState::new();

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let (new_s, _) = sha1(&buf[..n], s);
        s = new_s;
    }

    let digest = sha1_final(s);

    if let Some(name) = name {
        println!("{}  {}", digest.iter().map(|b| format!("{:02x}", b)).collect::<String>(), name);
    } else {
        println!("{}", digest.iter().map(|b| format!("{:02x}", b)).collect::<String>());
    }

    Ok(())
}

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let mut files = Vec::new();

    argument! {
        args: args,
        options: {},
        command: |arg| files.push(PathBuf::from(OsStr::from_bytes(arg))),
        on_invalid: |arg| usage!("sha1sum: invalid option -- '{}'", arg as char)
    }

    if files.is_empty() {
        if let Err(e) = sum(io::stdin(), None) {
            error!("sha1sum: {}", e);
        }
    } else {
        for file in files {
            match File::open(&file) {
                Ok(f) => {
                    if let Err(e) = sum(f, Some(file.to_str().unwrap())) {
                        error!("sha1sum: {}: {}", file.display(), e);
                    }
                }
                Err(e) => error!("sha1sum: {}: {}", file.display(), e),
            }
        }
    }
}
