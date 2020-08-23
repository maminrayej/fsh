use rudac::queue::Circular;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process;

// Manages history of shell
pub struct History {
    // Path to history file which is /home/username/.fsh/history
    history_file_path_buf: Option<PathBuf>,

    // A circular buffer to store executed commands
    history_buffer: Circular<String>,
}

impl History {
    // Init a new history handler
    fn init(history_file_path_buf: Option<PathBuf>) -> Self {
        // Create a history handler with buffer of size 1000 commands
        let mut history_handler = History {
            history_file_path_buf,
            history_buffer: Circular::new(1000),
        };

        // If some history file is provided, read commands from file and put it in the history buffer
        if history_handler.history_file_path_buf.is_some() {
            let history_file = File::open(history_handler.history_file_path_ref()).unwrap();
            let commands = BufReader::new(history_file).lines();
            for command in commands {
                if let Ok(command_text) = command {
                    history_handler.add_command(command_text);
                }
            }
        }

        return history_handler;
    }

    // Get a reference to history file path
    fn history_file_path_ref(&self) -> &PathBuf {
        self.history_file_path_buf.as_ref().unwrap()
    }

    // Get a mutable reference to history file path
    fn history_file_path_mut_ref(&mut self) -> &mut PathBuf {
        self.history_file_path_buf.as_mut().unwrap()
    }

    // Add a command to history buffer
    pub fn add_command(&mut self, command: String) {
        self.history_buffer.enqueue(command)
    }

    // Get all commands in history buffer
    pub fn get_history_elements(&self) -> Vec<&String> {
        self.history_buffer.into_iter().collect()
    }

    // Search for command in history buffer and return suggestions
    pub fn search(&self, command: &String) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Suggest any command that start with specified `command`
        if command.trim().len() != 0 {
            for element in self.history_buffer.into_iter() {
                if element.starts_with(command) {
                    suggestions.push(element.clone());
                }
            }
        }

        // Reverse the order the suggestion to put latest found match at index 0
        suggestions.reverse();

        return suggestions;
    }

    // Return history element at specified `index`
    pub fn get(&self, index: usize) -> Option<&String> {
        if index < self.history_buffer.size() {
            return Some(&self.history_buffer[self.history_buffer.size() - 1 - index]);
        }

        None
    }

    // Save history buffer to history file
    pub fn save(&mut self) {
        // If path to history file is specified, save the buffer
        if self.history_file_path_buf.is_some() {
            // Convert Vec<&String> to Vev<String>
            let history_elements: Vec<String> = self
                .get_history_elements()
                .into_iter()
                .map(|element| element.clone())
                .collect();

            let history_file_path_buf = self.history_file_path_mut_ref();

            fs::write(history_file_path_buf, history_elements.join("\n")).unwrap();
        }
    }
}

// Create a history handler and return it
pub fn get_history_handler() -> History {
    // If history file could be found or created
    if let Ok(history_file_path_buf) = get_history_file() {
        History::init(Some(history_file_path_buf))
    } else {
        History::init(None)
    }
}

// Return path to history file
fn get_history_file() -> std::result::Result<PathBuf, &'static str> {
    // Launch whoami program and collect its output
    if let Ok(output) = process::Command::new("whoami").output() {
        // Output of whoami program is the username
        let username = std::str::from_utf8(&output.stdout)
            .unwrap()
            .to_string()
            .trim()
            .to_string();

        // History file is in /home/{username}/.fsh/history
        let fsh_path_str = format!("/home/{username}/.fsh", username = username);
        let fsh_path = Path::new(&fsh_path_str);

        if !fsh_path.exists() {
            // Create fsh dir
            if let Ok(_) = fs::create_dir(fsh_path) {
                // Create history file
                let history_file_path = fsh_path.join("history");
                if let Ok(_) = File::create(&history_file_path) {
                    return Ok(history_file_path);
                } else {
                    println!("Could not create history file.")
                }
            } else {
                println!("Could not create .fsh dir in user home directory.");
            }
        } else {
            let history_file_path = fsh_path.join("history");

            if !history_file_path.exists() {
                if let Ok(_) = File::create(&history_file_path) {
                    return Ok(history_file_path);
                } else {
                    println!("Could not create history file.")
                }
            }

            return Ok(history_file_path);
        }
    }

    return Err("Failed to get history file");
}
