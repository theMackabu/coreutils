#[cfg(not(feature = "start"))]
#[macro_export]
macro_rules! start {
    ($ident:ident, $argc:expr, $argv:expr) => {
        $ident::_start($argc - 1, unsafe { $argv.offset(1) })
    };
}

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
