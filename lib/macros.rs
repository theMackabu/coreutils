#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        std::process::exit(1)
    }};
}

#[macro_export]
macro_rules! usage {
    () => {{
        eprintln!("{USAGE}");
        std::process::exit(1)
    }};
    ($msg:expr) => {{
        eprintln!("{}\n{USAGE}", $msg);
        std::process::exit(1)
    }};
    ($msg:expr, $code:expr) => {{
        eprintln!("{}", $msg);
        std::process::exit($code)
    }};
}

#[macro_export]
macro_rules! argument {
    (
        args: $args:expr,
        options: { $( $opt:ident => $set:expr ),* },
        command: $command:expr,
        on_invalid: $on_invalid:expr
    ) => {
        for arg in $args {
            if arg.starts_with(b"-") && arg.len() > 1 {
                for &byte in &arg[1..] {
                    match byte {
                        $(b if b == stringify!($opt).as_bytes()[0] => $set, )*
                        _ => { $on_invalid(byte as char) }
                    }
                }
            } else { $command(arg) }
        }
    };
}

#[cfg(not(feature = "bin"))]
#[macro_export]
macro_rules! entry {
    (args: { $( $key:ident $( : $value:ident )? ),* }, commands: [$($cmd:ident),+ $(,)?], fallback: $command:expr) => {
        let mut s_args = Args { $($key $( : $value )?,)* };

        if s_args.program == b"core" {
            if s_args.args.is_empty() {
                usage!();
            }
            s_args.program = s_args.args[0];
        }

        match str::from_utf8(s_args.program) {
            $(Ok(stringify!($cmd)) => match s_args.caller {
                b"core" => $cmd::_start(s_args.argc - 1, unsafe { s_args.argv.offset(1) }),
                _ => $cmd::_start(s_args.argc, s_args.argv),
            },)*
            fallback_cmd => $command(fallback_cmd.unwrap_or("?")),
        }
    };
}
