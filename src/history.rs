use rudac::queue::Circular;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process;

pub struct History {
    history_file_path_buf: Option<PathBuf>,
    history_buffer: Circular<String>,
}

impl History {
    fn init(history_file_path_buf: Option<PathBuf>) -> Self {
        let mut history_handler = History {
            history_file_path_buf,
            history_buffer: Circular::new(1000),
        };

        // Read commands from history file
        let mut commands_vec = vec![];

        if history_handler.history_file_path_buf.is_some() {
            let history_file =
                File::open(history_handler.history_file_path_buf.as_ref().unwrap()).unwrap();
            let commands = BufReader::new(history_file).lines();
            for command in commands {
                if let Ok(command_text) = command {
                    // println!("Adding command... {}", &command_text);
                    commands_vec.push(command_text);
                }
            }
        }

        for command in commands_vec {
            history_handler.add_command(command);
        }

        return history_handler;
    }

    pub fn add_command(&mut self, command: String) {
        self.history_buffer.enqueue(command)
    }

    pub fn get_history_elements(&self) -> Vec<String> {
        let mut history_elements = vec![];

        for element in self.history_buffer.into_iter() {
            history_elements.push(element.clone());
        }

        return history_elements;
    }

    pub fn search(&self, command: &String) -> Vec<String> {
        if command.trim().len() == 0 {
            return Vec::new();
        }

        let mut found_matches = Vec::new();

        for element in self.history_buffer.into_iter() {
            if element.starts_with(command) {
                found_matches.push(element.clone());
            }
        }

        found_matches.reverse();

        return found_matches;
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        if index >= self.history_buffer.size() {
            None
        } else {
            Some(&self.history_buffer[self.history_buffer.size() - 1 - index])
        }
    }

    pub fn save(&mut self) {
        if self.history_file_path_buf.is_some() {
            let history_file_path_buf = self.history_file_path_buf.as_mut().unwrap();
            let mut history_elements = vec![];

            for element in self.history_buffer.into_iter() {
                history_elements.push(element.clone());
            }

            fs::write(history_file_path_buf, history_elements.join("\n")).unwrap();
        }
    }
}

pub(crate) fn get_history_handler() -> History {
    if let Ok(history_file_path_buf) = get_history_file() {
        History::init(Some(history_file_path_buf))
    } else {
        History::init(None)
    }
}

fn get_history_file() -> std::result::Result<PathBuf, &'static str> {
    // Launch whoami program and collect its output
    if let Ok(output) = process::Command::new("whoami").output() {
        // Output of whoami program is the username
        let username = std::str::from_utf8(&output.stdout)
            .unwrap()
            .to_string()
            .trim()
            .to_string();

        // history file is in /home/{username}/.fsh/history
        let fsh_path_str = format!("/home/{username}/.fsh", username = username);
        let fsh_dir = Path::new(&fsh_path_str);

        if !fsh_dir.exists() {
            // create fsh dir
            if let Ok(_) = fs::create_dir(fsh_dir) {
                return Ok(fsh_dir.join("history"));
            } else {
                println!("Could not create .fsh dir in user home directory. Accessing history file failed.");
            }
        } else {
            let history_file_path = fsh_dir.join("history");

            if !history_file_path.exists() {
                File::create(fsh_dir.join("history")).unwrap();
            }

            return Ok(fsh_dir.join("history"));
        }
    }

    return Err("Failed to get history file");
}
