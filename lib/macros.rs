#[macro_export]
macro_rules! module {
    ($($name:ident),+) => {
        $(mod $name;)+

        fn modules() -> &'static [(&'static str, fn(argc: isize, argv: *const *const u8) -> isize)] {
            &[$((stringify!($name), $name::main),)+]
        }

        fn init_commands() -> &'static [(&'static str, &'static str)] {
            &[$((stringify!($name), $name::DESCRIPTION),)+]
        }
    };
}

#[macro_export]
macro_rules! utf8_n {
    ($opt:expr) => {
       String::from_utf8_lossy($opt).into_owned()
    };
    (some->$opt:expr) => {
       Some(String::from_utf8_lossy($opt).into_owned())
    };
}

#[macro_export]
macro_rules! stdout {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        std::process::exit(1)
    }};
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
        eprintln!("{}", &*USAGE);
        std::process::exit(1)
    }};
    (help->core) => {{
        println!("{}\n", &*USAGE);
        println!("{}", options());
        std::process::exit(0)
    }};
    (help->$) => {{
        println!("{}", &*DESCRIPTION);
        println!("{}", &*USAGE);
        std::process::exit(0)
    }};
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        eprintln!("{}", &*USAGE);
        std::process::exit(1)
    }};
    ($code:expr, $($arg:tt)*) => {{
        eprintln!($($arg)*);
        eprintln!("{}", &*USAGE);
        std::process::exit($code)
    }};
}

#[macro_export]
macro_rules! argument {
    (
        $args:expr,
        flags: { $( $flag:ident => $set:expr ),* },
        options: { $( $opt:ident => $func:expr ),* },
        command: $command:expr,
        on_invalid: $on_invalid:expr
    ) => {
        let mut iter = $args.into_iter();
        while let Some(arg) = iter.next() {
            if arg.starts_with(b"-") && arg.len() > 1 {
                for &byte in &arg[1..] {
                    match byte {
                        $(b if b == stringify!($flag).as_bytes()[0] => $set, )*
                        $(b if b == stringify!($opt).as_bytes()[0] => {
                            if let Some(next_arg) = iter.next() {
                                $func(next_arg as &[u8]);
                            } else {
                                $on_invalid(byte as char);
                            }
                        }, )*
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
    (args: { $( $key:ident $( : $value:ident )? ),* }, options: {
        $( $short_opt:ident | $long_opt:ident: $opt_desc:expr => $opt_action:expr ),* $(,)?
    }) => {
        let mut s_args = Args { $($key $( : $value )?,)* };

        if s_args.program == b"core" {
            if s_args.args.is_empty() {
                usage!(help->core)
            }
            s_args.program = s_args.args[0];
        }

        let program_str = str::from_utf8(s_args.program).unwrap_or("?");
        let dispatch_table: &[(&str, fn(argc: isize, argv: *const *const u8) -> isize)] = modules();

        if let Some(&(_, start_fn)) = dispatch_table.iter().find(|&&(cmd_name, _)| cmd_name == program_str) {
            match s_args.caller {
                b"core" => start_fn(s_args.argc - 1, unsafe { s_args.argv.offset(1) }),
                _ => start_fn(s_args.argc, s_args.argv),
            }
        } else {
            match program_str {
                $(concat!("-", stringify!($short_opt)) => $opt_action,)*
                $(concat!("--", stringify!($long_opt)) => $opt_action,)*
                cmd => error!("core: '{cmd}' is not a option. See 'core --help'."),
            }
        };

        fn options() -> String {
            let mut usage_str = String::from("Options:\n");
            let mut option_lines = Vec::new();

            let max_opt_len = {
                let short_opts = vec![$(format!("-{}", stringify!($short_opt)),)*];
                let long_opts = vec![$(format!("--{}", stringify!($long_opt)),)*];
                short_opts.iter().chain(long_opts.iter()).map(|opt| opt.len()).max().unwrap_or(0)
            };

            $(
                let short_opt = format!("-{}", stringify!($short_opt));
                let long_opt = format!("--{}", stringify!($long_opt));
                option_lines.push(format!("  {:<width$}  {}", format!("{}, {}", short_opt, long_opt), $opt_desc, width = max_opt_len + 2));
            )*

            usage_str.tap(|s| s.push_str(&option_lines.join("\n")))
        }
    };
}

#[macro_export]
macro_rules! lazy_lock {
    ($(#[$attr:meta])* static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        $crate::__lazy_lock_internal!($(#[$attr])* () static $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        $crate::__lazy_lock_internal!($(#[$attr])* (pub) static $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        $crate::__lazy_lock_internal!($(#[$attr])* (pub ($($vis)+)) static $N : $T = $e; $($t)*);
    };
    () => ()
}

#[macro_export]
macro_rules! __lazy_lock_internal {
    ($(#[$attr:meta])* ($($vis:tt)*) static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        $(#[$attr])*
        $($vis)* static $N: std::sync::LazyLock<$T> = std::sync::LazyLock::new(|| $e);
        $crate::lazy_lock!($($t)*);
    };
    () => ()
}

#[macro_export]
macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}
