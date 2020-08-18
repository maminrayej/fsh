use std::env;
use termion::{color, style};
use regex::Regex;
use std::process;

// Returns a prompt and its length without any style/color
pub(crate) fn prompt() -> (String, usize) {
    // Get current working directory
    let mut current_path = String::new();
    if let Ok(path) = env::current_dir() {
        current_path = path.to_str().unwrap().to_string();
    }

    // Get system username of the user
    let mut username = String::new();
    // Launch whoami program and collect its output
    if let Ok(output) = process::Command::new("whoami").output() {
        // Output of whoami program is the username
        username = std::str::from_utf8(&output.stdout).unwrap().to_string().trim().to_string();
    }

    // Replace home directory with special and short character
    let home_dir_regex = format!("/home/{username}/|/home/{username}", username=username);
    let re = Regex::new(&home_dir_regex).unwrap();
    let current_path = re.replace(&current_path, "⌂|");

    let current_path_len = current_path.chars().count();

    let formatted_prompt = format!(
        "{current_path_style}{current_path_color}{current_path}{style_reset}{color_reset}$ ",
        current_path = current_path,
        current_path_style = style::Bold,
        current_path_color = color::Fg(color::Green),
        style_reset = style::Reset,
        color_reset = color::Fg(color::Reset)
    );

    let dollar_sign_len = 1;
    let final_space_len = 1;

    return (
        formatted_prompt,
        current_path_len + dollar_sign_len + final_space_len,
    );
}
