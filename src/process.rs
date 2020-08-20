use glob::glob;
use std::env;
use std::process::Command;
use std::str;

pub(crate) fn execute(command_line: String) {
    // Tokenize command line
    let mut tokens: Vec<&str> = command_line
        .rsplit(|c| c == ' ' || c == '\t' || c == '\r')
        .rev()
        .collect();

    let args = expand_arguments(tokens.split_off(1));
    let command = tokens.pop().unwrap();

    match command {
        "pwd" => pwd(),
        "cd" => cd(args),
        "echo" => echo(args),
        "exit" => exit(),
        _ => launch_command(command, args),
    }
}

fn launch_command(command: &str, args: Vec<String>) {
    if let Ok(child) = Command::new(command).args(args).spawn() {
        let output = child.wait_with_output().unwrap();
        let _output = str::from_utf8(&output.stdout).unwrap();

        print!("{}", _output);
    } else {
        println!("Something went wrong")
    }
}

// built in commands
fn pwd() {
    if let Ok(path) = env::current_dir() {
        println!("{}\r", path.display());
    } else {
        println!("PWD command faced issue")
    }
}

fn echo(args: Vec<String>) {
    println!("{}\r", args.join(" "));
}

fn exit() {
    std::process::exit(0)
}

fn cd(args: Vec<String>) {
    if args.len() > 0 {
        env::set_current_dir(&args[0]).unwrap();
    } else {
        let mut username = String::new();
        if let Ok(output) = Command::new("whoami").output() {
            username = std::str::from_utf8(&output.stdout).unwrap().to_string();
        }
        env::set_current_dir(&format!("/home/{}", username.trim())).unwrap();
    }
}

fn expand_arguments<'a>(args: Vec<&str>) -> Vec<String> {
    let mut expanded_args = Vec::new();

    for arg in args {
        if arg.contains('*') {
            let path = std::path::Path::new(arg);
            let paths = if path.is_relative() {
                glob(&format!("./{}", arg)).expect("Failed to read glob pattern")
            } else {
                glob(arg).expect("Failed to read glob pattern")
            };

            for path in paths {
                expanded_args.push(path.unwrap().as_path().to_str().unwrap().to_string());
            }
        } else {
            expanded_args.push(arg.to_string());
        }
    }

    return expanded_args;
}
