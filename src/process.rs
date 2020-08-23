use crate::history::History;
use glob::glob;
use std::env;
use std::process::Command;
use std::str;

pub(crate) fn execute(command_line: String, history_handler: &mut History) {
    // Split command line
    // [0] is command and [1..] are arguments
    let mut tokens: Vec<&str> = command_line
        .rsplit(|c| c == ' ' || c == '\t' || c == '\r')
        .rev()
        .collect();

    // Expand arguments if they contain glob like: *.mp3
    let args = expand_arguments(tokens.split_off(1));

    let command = tokens.pop().unwrap();

    match command {
        "pwd" => pwd(),
        "cd" => cd(args),
        "echo" => echo(args),
        "exit" => {
            history_handler.save();
            exit();
        }
        "history" => history(history_handler.get_history_elements()),
        _ => launch_command(command, args), // External command
    }
}

// Execute an external command/program and display its output
fn launch_command(command: &str, args: Vec<String>) {
    if let Ok(child) = Command::new(command).args(args).spawn() {
        let output = child.wait_with_output().unwrap();
        print!("{}", str::from_utf8(&output.stdout).unwrap());
    } else {
        println!("Something went wrong")
    }
}

// Built in commands
fn pwd() {
    if let Ok(path) = env::current_dir() {
        println!("{}", path.display());
    }
}

fn echo(args: Vec<String>) {
    println!("{}", args.join(" "));
}

fn exit() {
    std::process::exit(0)
}

fn cd(args: Vec<String>) {
    // If a path is provided, change directory else change directory to home directory
    if args.len() > 0 {
        // If directory does not exist, print error message
        if let Err(msg) = env::set_current_dir(&args[0]) {
            println!("{}", msg);
        }
    } else {
        let mut username = String::new();
        if let Ok(output) = Command::new("whoami").output() {
            username = std::str::from_utf8(&output.stdout).unwrap().to_string();
        }
        env::set_current_dir(&format!("/home/{}", username.trim())).unwrap();
    }
}

fn history(history_elements: Vec<&String>) {
    for element in history_elements {
        println!("{}", element)
    }
}

// Expand each argument that contains glob like: *.mp3
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
