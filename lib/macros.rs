#[cfg(not(feature = "bin"))]
#[macro_export]
macro_rules! start {
    ($ident:ident, $args:expr) => match $args.caller {
        b"core" => $ident::_start($args.argc - 1, unsafe { $args.argv.offset(1) }),
        _ => $ident::_start($args.argc, $args.argv),
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
        std::process::exit(1)
    };
}

#[macro_export]
macro_rules! usage {
    () => {
        eprintln!("{USAGE}");
        std::process::exit(1)
    };
    ($msg:expr) => {
        eprintln!("{}\n{USAGE}", $msg);
        std::process::exit(1)
    };
    ($msg:expr, $code:expr) => {
        eprintln!("{}", $msg);
        std::process::exit($code)
    };
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
