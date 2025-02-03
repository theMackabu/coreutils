#![cfg_attr(feature = "bin", no_main)]

extern crate entry;

use std::io::{Read, Write};
use std::net::TcpStream;

const USAGE: &str = "usage: http [options] <url>";
pub const DESCRIPTION: &str = "Make basic HTTP requests.";

fn parse_url(url: &str) -> Result<(String, String, u16, String), Box<dyn Error>> {
    let parts: Vec<&str> = url.splitn(2, "://").collect();
    if parts.len() != 2 {
        return Err("Invalid URL format".into());
    }

    let scheme = parts[0];
    let rest = parts[1];

    let (host, path) = rest.split_once('/').unwrap_or((rest, "/"));

    let (host, port) = if let Some((h, p)) = host.rsplit_once(':') {
        (h, p.parse().unwrap_or(80))
    } else {
        (host, if scheme == "https" { 443 } else { 80 })
    };

    Ok((scheme.to_string(), host.to_string(), port, format!("/{}", path)))
}

fn send_request(host: &str, port: u16, path: &str) -> Result<String, Box<dyn Error>> {
    let mut stream = TcpStream::connect((host, port))?;

    let request = format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: coreutils-http/0.1\r\n\
         Connection: close\r\n\
         \r\n",
        path, host
    );

    stream.write_all(request.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    Ok(response)
}

#[entry::gen("bin", "mut", "safe")]
fn entry() -> ! {
    let url = args.next().unwrap_or_else(|| usage!("http: missing URL"));
    let url = String::from_utf8_lossy(url).to_string();

    match parse_url(&url) {
        Ok((scheme, host, port, path)) => {
            if scheme != "http" {
                error!("http: only HTTP scheme is supported");
            }

            match send_request(&host, port, &path) {
                Ok(response) => {
                    print!("{}", response);
                }
                Err(e) => error!("http: request failed: {}", e),
            }
        }
        Err(e) => error!("http: {}", e),
    }
}
