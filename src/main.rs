mod read;
mod process;
mod preprocess;

fn main() {
    read::read_loop();
}

// fn fsh_loop() {
//     print_prefix();
//     while let Some(line) = fsh_read_line() {
//         let trimmed_line = line.trim().to_owned();
//         let tokens = fsh_split_line(&trimmed_line);

//         // println!("{:?}", tokens);
//         fsh_execute(tokens);
//         print_prefix();
//     }
// }

// fn print_prefix() {
//     let mut current_path = String::new();
//     if let Ok(path) = env::current_dir() {
//         current_path = path.to_str().unwrap().to_string();
//     }

//     let mut username = String::new();
//     if let Ok(output) = process::Command::new("whoami").output() {
//         username = std::str::from_utf8(&output.stdout).unwrap().to_string();
//     }

//     // ⌂
//     current_path = current_path.replace(&format!("/home/{}/", username.trim()), "⌂|");
//     current_path = current_path.replace(&format!("/home/{}", username.trim()), "⌂|");

//     print!(
//         "{username_color}{username_style}{username}{reset_color}{path_color}{current_path}{reset_style}{reset_color}$ ",
//         username_color = color::Fg(color::Blue),
//         username_style = style::Bold,
//         username = "",//username.trim(),
//         path_color = color::Fg(color::Green),
//         current_path = current_path.trim(),
//         reset_color = color::Fg(color::Reset),
//         reset_style = style::Reset
//     );
//     io::stdout().flush().unwrap();
// }

// fn fsh_read_line() -> Option<String> {
//     let mut input = String::new();

//     match io::stdin().read_line(&mut input) {
//         Ok(_) => {
//             // println!("Read {} bytes", n);
//             return Some(input);
//         }
//         Err(error) => {
//             println!("Failed to read from stdin: {}", error);
//             return None;
//         }
//     }
// }

// fn fsh_split_line(line: &String) -> Vec<&str> {
//     line.rsplit(|c| c == ' ' || c == '\t' || c == '\r')
//         .rev()
//         .collect()
// }

// fn fsh_execute(mut tokens: Vec<&str>) {
//     let args = tokens.split_off(1);
//     let command = tokens.pop().unwrap();

//     match command {
//         "pwd" => pwd(),
//         "cd" => cd(args),
//         _ => fsh_launch(command, args),
//     }
// }

// fn fsh_launch(command: &str, mut args: Vec<&str>) {
//     if command == "ls" {
//         args.push("--color");
//     }
//     if let Ok(mut child) = process::Command::new(command).args(args).spawn() {
//         child.wait().unwrap();
//     } else {
//         println!("Something went wrong")
//     }
// }

// // built in commands
// fn pwd() {
//     if let Ok(path) = env::current_dir() {
//         println!("{}", path.display());
//     } else {
//         println!("PWD command faced issue")
//     }
// }

// fn cd(args: Vec<&str>) {
//     if args.len() > 0 {
//         env::set_current_dir(args[0]).unwrap();
//     } else {
//         let mut username = String::new();
//         if let Ok(output) = process::Command::new("whoami").output() {
//             username = std::str::from_utf8(&output.stdout).unwrap().to_string();
//         }
//         env::set_current_dir(&format!("/home/{}", username.trim())).unwrap();
//         // println!("{}", format!("/home/{}", username.trim()));
//     }
// }
