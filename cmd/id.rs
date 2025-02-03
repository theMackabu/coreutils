#![cfg_attr(feature = "bin", no_main)]

extern crate entry;
extern crate uid;

use self::uid::*;
use std::io;

const USAGE: &str = "usage: id [-u] [-g] [-G] [-n] [user]";
pub const DESCRIPTION: &str = "Print user and group information";

fn print_id(options: &IdOptions, username: Option<&str>, group: Option<&str>) -> io::Result<()> {
    let (uid, gid, name) = get_user_info(username, group)?;

    if options.print_user {
        if options.use_name {
            println!("{}", name);
        } else {
            println!("{}", uid);
        }
    } else if options.print_group {
        if options.use_name {
            println!("{}", get_group_name(gid)?);
        } else {
            println!("{}", gid);
        }
    } else if options.print_groups {
        let groups = get_groups()?;
        if options.use_name {
            let group_names: Vec<String> = groups.iter().filter_map(|&gid| get_group_name(gid).ok()).collect();
            println!("{}", group_names.join(" "));
        } else {
            let group_ids: Vec<String> = groups.iter().map(|&gid| gid.to_string()).collect();
            println!("{}", group_ids.join(" "));
        }
    } else {
        print!("uid={}({}) gid={}({})", uid, name, gid, get_group_name(gid)?);
        let groups = get_groups()?;
        if !groups.is_empty() {
            print!(" groups=");
            for (i, &group) in groups.iter().enumerate() {
                if i > 0 {
                    print!(",");
                }
                print!("{}({})", group, get_group_name(group)?);
            }
        }
        println!();
    }

    Ok(())
}

#[entry::gen("bin", "mut", "safe")]
fn entry() -> ! {
    let mut username = None;
    let mut group = None;

    let mut options = IdOptions {
        print_user: false,
        print_group: false,
        print_groups: false,
        use_name: false,
    };

    argument! {
        args: args.to_owned(),
        options: {
            u => options.print_user = true,
            g => {
                args.next();
                if let Some(arg) = args.next() {
                    group = Some(String::from_utf8_lossy(arg).into_owned());
                }
                options.print_group = true
            },
            G => options.print_groups = true,
            n => options.use_name = true
        },
        command: |arg| {
            if username.is_some() {
                usage!("id: too many arguments");
            }
            username = Some(String::from_utf8_lossy(arg).into_owned());
        },
        on_invalid: |arg| usage!("id: invalid option -- '{}'", arg as char)
    }

    if let Err(e) = print_id(&options, username.as_deref(), group.as_deref()) {
        error!("id: {}", e);
    }
}
