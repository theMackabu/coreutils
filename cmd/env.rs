#![cfg_attr(feature = "bin", feature(start))]

extern crate entry;
extern crate env;

use std::{collections::HashMap, str::from_utf8};

const USAGE: &str = "usage: env [-i] [NAME=VALUE]... [COMMAND [ARG]...]";
pub const DESCRIPTION: &str = "Set the environment for command invocation";

#[entry::gen("bin", "safe")]
fn entry() -> ! {
    let env = env::vars().into_iter();

    let mut ignore_env = false;
    let mut c_args: Vec<&str> = args
        .to_owned()
        .map(|arg| from_utf8(arg).unwrap_or("?"))
        .collect();

    let mut env_vars: HashMap<String, String> = env
        .filter_map(|line| {
            let mut parts = line.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            if !key.is_empty() {
                Some((key.to_string(), value.to_string()))
            } else {
                None
            }
        })
        .collect();

    argument! {
        args: args,
        options: {
            i => ignore_env = true
        },
        command: |arg: &[u8]| {
            let arg = from_utf8(arg).unwrap_or("?");
            if let Some(pos) = arg.find('=') {
                let (key, value) = arg.split_at(pos);
                env_vars.insert(key.to_string(), value.to_string());
            }
        },
        on_invalid: |arg| usage!("env: invalid option -- '{}'", arg as char)
    }

    if ignore_env {
        env_vars.clear();
    }

    if c_args.is_empty() {
        for (key, value) in &env_vars {
            println!("{}={}", key, value);
        }
    } else {
        let command = c_args.remove(0);
        let mut command = std::process::Command::new(command);
        command.args(&c_args).envs(&env_vars);

        match command.status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(e) => error!("env: {}", e),
        }
    }
}
