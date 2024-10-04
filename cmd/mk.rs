#![cfg_attr(feature = "bin", feature(start))]

extern crate entry;

use std::collections::HashMap;
use std::process::Command;

const USAGE: &str = "usage: mk [-f mkfile] ... [ option ... ] [ target ... ]";
pub const DESCRIPTION: &str = "Maintain make (plan9) related files";

#[derive(Clone)]
struct Rule {
    targets: Vec<String>,
    prerequisites: Vec<String>,
    recipe: Vec<String>,
}

struct MkOptions {
    mkfile: String,
    assume_out_of_date: bool,
    debug: bool,
    explain: bool,
    force_intermediate: bool,
    keep_going: bool,
    print_only: bool,
    sequential: bool,
    touch: bool,
}

fn parse_mkfile(content: &str) -> HashMap<String, Rule> {
    let mut rules = HashMap::new();
    let mut current_rule: Option<Rule> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if !line.starts_with('\t') {
            if let Some(rule) = current_rule.take() {
                for target in &rule.targets {
                    rules.insert(target.clone(), rule.clone());
                }
            }

            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
            if parts.len() == 2 {
                let targets: Vec<String> = parts[0].split_whitespace().map(String::from).collect();
                let prerequisites: Vec<String> = parts[1].split_whitespace().map(String::from).collect();
                current_rule = Some(Rule {
                    targets,
                    prerequisites,
                    recipe: Vec::new(),
                });
            }
        } else if let Some(ref mut rule) = current_rule {
            rule.recipe.push(line[1..].to_string());
        }
    }

    if let Some(rule) = current_rule {
        for target in &rule.targets {
            rules.insert(target.clone(), rule.clone());
        }
    }

    rules
}

fn execute_recipe(rule: &Rule, options: &MkOptions) -> io::Result<()> {
    if options.print_only {
        for line in &rule.recipe {
            println!("{}", line);
        }
    } else {
        let shell = std::env::var("MKSHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let recipe = rule.recipe.join("\n");
        let status = Command::new(shell).arg("-c").arg(&recipe).status()?;

        if !status.success() && !options.keep_going {
            return Err(io::Error::new(io::ErrorKind::Other, "Recipe execution failed"));
        }
    }
    Ok(())
}

fn build_target(target: &str, rules: &HashMap<String, Rule>, options: &MkOptions, built: &mut HashMap<String, bool>) -> io::Result<()> {
    if *built.get(target).unwrap_or(&false) {
        return Ok(());
    }

    if let Some(rule) = rules.get(target) {
        for prereq in &rule.prerequisites {
            build_target(prereq, rules, options, built)?;
        }

        if options.explain {
            println!("Building target: {}", target);
        }

        execute_recipe(rule, options)?;
        built.insert(target.to_string(), true);
    } else if !Path::new(target).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("No rule to make target '{}'", target)));
    }

    Ok(())
}

#[entry::gen("bin", "mut", "safe")]
fn entry() -> ! {
    let mut options = MkOptions {
        mkfile: "mkfile".into(),
        assume_out_of_date: false,
        debug: false,
        explain: false,
        force_intermediate: false,
        keep_going: false,
        print_only: false,
        sequential: false,
        touch: false,
    };
    let mut targets = Vec::new();

    argument! {
        args: args.to_owned(),
        options: {
            f => {
                args.next();
                options.mkfile = String::from_utf8_lossy(args.next().unwrap_or_else(|| usage!("mk: option requires an argument -- 'f'"))).into_owned();
            },
            a => options.assume_out_of_date = true,
            d => options.debug = true,
            e => options.explain = true,
            i => options.force_intermediate = true,
            k => options.keep_going = true,
            n => options.print_only = true,
            s => options.sequential = true,
            t => options.touch = true
        },
        command: |arg| targets.push(String::from_utf8_lossy(arg).into_owned()),
        on_invalid: |arg| usage!("mk: invalid option -- '{}'", arg as char)
    }

    let mkfile_content = std::fs::read_to_string(&options.mkfile).unwrap_or_else(|_| error!("mk: cannot read mkfile '{}'", options.mkfile));

    let rules = parse_mkfile(&mkfile_content);

    if targets.is_empty() {
        if let Some(first_rule) = rules.values().next() {
            targets.push(first_rule.targets[0].clone());
        } else {
            error!("mk: no targets");
        }
    }

    let mut built = HashMap::new();
    for target in targets {
        if let Err(e) = build_target(&target, &rules, &options, &mut built) {
            error!("mk: {}", e);
        }
    }
}
