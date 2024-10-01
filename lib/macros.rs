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

#[macro_export]
macro_rules! argument {
    ($args:expr, $options:expr, $files:expr, $match:expr, $else:expr) => {
        for arg in $args {
            if arg.starts_with(b"-") && arg.len() > 1 {
                for &byte in &arg[1..] {
                    $match(byte);
                }
            } else {
                $else(arg);
            }
        }
    };
}
