use crate::history::History;
use crate::preprocess::prompt;
use crate::process::execute;
use glob::glob;
use std::io::{stdin, stdout, Write};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::style;

pub fn read_loop(mut history_handler: History) {
    // Get the standard input stream
    let stdin = stdin();

    // Get the standard output stream and go to raw mode.
    let mut _stdout = stdout().into_raw_mode().unwrap();

    // This buffer contains user input
    // After every Enter this buffer will be collected and processed in order to run the command
    // This buffer will be updated after every insert and delete
    let mut char_buf = Vec::<char>::new();

    let mut suggestions = Vec::<String>::new();
    let mut suggestion_index = 0;
    let mut search_mode = false;

    let mut history_index: i64 = -1;

    // The print_prompt method prints the prompt and returns the minimum x coordinate that cursor can hold
    // Cursor can not go behind the prompt therefore this minimum value is the size of the printed prompt
    let mut min_cursor_x_bound = print_prompt(&mut _stdout);

    for c in stdin.keys() {
        let (cursor_x, cursor_y) = _stdout.cursor_pos().unwrap();

        match c.unwrap() {
            Key::Char('\n') => {
                if search_mode {
                    search_mode = false;
                    suggestions.clear();
                    suggestion_index = 0;

                    // Go to a new line
                    println!("\r");

                    // Print the prompt
                    print_prompt(&mut _stdout);

                    // Print the command
                    let command: String = char_buf.iter().collect();
                    print!("{}", command);
                    _stdout.flush().unwrap();
                }
                // Go to a new line
                println!("\r");

                // Execute the command in buffer
                if !char_buf.is_empty() {
                    // Exit of raw mode
                    std::mem::drop(_stdout);

                    // Extract command
                    let command: String = char_buf.iter().collect();

                    // Add command to history
                    history_handler.add_command(command.clone());

                    // Execute the command with normal tty
                    execute(command, &history_handler);

                    // Go into raw mode again
                    _stdout = stdout().into_raw_mode().unwrap();

                    char_buf.clear();
                }

                // Clear the current line for prompt
                write!(_stdout, "{}\r", termion::clear::CurrentLine).unwrap();
                _stdout.flush().unwrap();

                // Print the prompt
                min_cursor_x_bound = print_prompt(&mut _stdout);
            }
            Key::Char('\t') => {
                if search_mode {
                    if let Some(suggestion) = suggestions.get(suggestion_index) {
                        char_buf.clear();

                        // Set suggestion as current command
                        for _char in suggestion.chars() {
                            char_buf.push(_char);
                        }

                        // Clear the current line for prompt
                        write!(_stdout, "{}\r", termion::clear::CurrentLine).unwrap();
                        _stdout.flush().unwrap();

                        // Print the prompt
                        min_cursor_x_bound = print_prompt(&mut _stdout);

                        // print the suggestion as current command
                        print!("{}", suggestion);
                        _stdout.flush().unwrap();

                        search_mode = false;
                        suggestions.clear();
                        suggestion_index = 0;
                    }
                } else {
                    let command: String = char_buf.iter().collect();

                    let mut _index = 0;
                    let cursor_index = (cursor_x - min_cursor_x_bound) as usize;
                    // first space before cursor
                    for (index, _char) in command
                        .chars()
                        .rev()
                        .skip(command.len() - cursor_index)
                        .enumerate()
                    {
                        if _char == ' ' {
                            _index = command.len() - ((command.len() - cursor_index) + index);
                            break;
                        }
                    }
                    // path glob
                    if _index != 0 {
                        let glob_text = &command[_index..cursor_index].trim();
                        let glob_entries = get_entries_of_glob(glob_text);
                        let mut suggestions = vec![];
                        for entry in glob_entries {
                            match entry {
                                Ok(suggestion) => suggestions.push(suggestion),
                                Err(_) => println!("Error occurred during glob search"),
                            }
                        }
                        // Fill the rest of user typed text if there is only one suggestion
                        if suggestions.len() == 1 {
                            let suggestion_str =
                                suggestions.pop().unwrap().to_str().unwrap().to_string();
                            for _char in suggestion_str.chars().skip(glob_text.chars().count()) {
                                char_buf.push(_char);
                            }
                            print!("{}", &suggestion_str[glob_text.chars().count()..]);
                            _stdout.flush().unwrap();
                        } else if suggestions.len() > 1 {
                            // Go to a new line
                            println!("\r");
                            // Print all suggestions
                            for path in suggestions {
                                println!("{}\r", path.as_path().display());
                            }
                            // Print the prompt
                            min_cursor_x_bound = print_prompt(&mut _stdout);
                            // Print the already typed command
                            print!("{}", command);
                            _stdout.flush().unwrap();
                        }
                    }
                }
            }
            Key::Char(c) => {
                char_buf.push(c);

                print!("{}", c);
                _stdout.flush().unwrap();

                if search_mode {
                    // already typed command
                    let command: String = char_buf.iter().collect();

                    suggestion_index = 0;

                    // get suggestions for this command
                    suggestions = history_handler.search(&command);

                    print_suggestion(
                        &command,
                        suggestions.get(suggestion_index),
                        cursor_y,
                        &mut _stdout,
                    );

                    min_cursor_x_bound = 5;
                }
            }
            Key::Ctrl(c) if c == 'c' => {
                println!("^C\r");

                if search_mode {
                    search_mode = false;
                    suggestions.clear();
                    suggestion_index = 0;
                }

                char_buf.clear();

                min_cursor_x_bound = print_prompt(&mut _stdout);
            }
            Key::Ctrl(c) if c == 'd' => {
                print!("exit");
                _stdout.flush().unwrap();

                // Save history to file
                history_handler.save();

                break;
            }
            Key::Ctrl(c) if c == 'r' => {
                // enable search mode
                search_mode = true;
                suggestion_index = 0;

                // already typed command
                let command: String = char_buf.iter().collect();

                // get suggestions for this command
                suggestions = history_handler.search(&command);

                print_suggestion(
                    &command,
                    suggestions.get(suggestion_index),
                    cursor_y,
                    &mut _stdout,
                );

                min_cursor_x_bound = 5;
            }
            Key::Left => move_cursor_left(&mut _stdout, cursor_x, cursor_y, min_cursor_x_bound),
            Key::Right => move_cursor_right(
                &mut _stdout,
                cursor_x,
                cursor_y,
                char_buf.len() as u16 + min_cursor_x_bound,
            ),
            Key::Up => {
                if search_mode {
                    // already typed command
                    let command: String = char_buf.iter().collect();

                    if let Some(_) = suggestions.get(suggestion_index + 1) {
                        suggestion_index += 1;
                    }

                    print_suggestion(
                        &command,
                        suggestions.get(suggestion_index),
                        cursor_y,
                        &mut _stdout,
                    );
                } else {
                    // println!("\n\rcurrent:{}\r", history_index);
                    history_index += 1;
                    if let Some(history_element) = history_handler.get(history_index as usize) {
                        // println!("\n\rup:{}\r", history_index);
                        char_buf.clear();
                        for _char in history_element.chars() {
                            char_buf.push(_char);
                        }

                        // clear current line
                        write!(_stdout, "\r{}", termion::clear::CurrentLine).unwrap();

                        // Print the prompt
                        min_cursor_x_bound = print_prompt(&mut _stdout);

                        // print the history element
                        print!("{}", history_element);
                        _stdout.flush().unwrap();
                    } else {
                        history_index -= 1;
                    }
                }
            }
            Key::Down => {
                if search_mode {
                    if suggestion_index == 0 {
                        continue;
                    }
                    // already typed command
                    let command: String = char_buf.iter().collect();

                    if let Some(_) = suggestions.get(suggestion_index - 1) {
                        suggestion_index -= 1;
                    }

                    print_suggestion(
                        &command,
                        suggestions.get(suggestion_index),
                        cursor_y,
                        &mut _stdout,
                    );
                } else {
                    if history_index <= 0 {

                        char_buf.clear();

                        // clear current line
                        write!(_stdout, "\r{}", termion::clear::CurrentLine).unwrap();

                        // Print the prompt
                        min_cursor_x_bound = print_prompt(&mut _stdout);

                        history_index = -1;

                        continue;
                    }

                    history_index -= 1;
                    if let Some(history_element) = history_handler.get(history_index as usize) {
                        // println!("\n\rdown:{}\r", history_index);

                        char_buf.clear();
                        for _char in history_element.chars() {
                            char_buf.push(_char);
                        }

                        // clear current line
                        write!(_stdout, "\r{}", termion::clear::CurrentLine).unwrap();

                        // Print the prompt
                        min_cursor_x_bound = print_prompt(&mut _stdout);

                        // print the history element
                        print!("{}", history_element);
                        _stdout.flush().unwrap();
                    }
                }
            }
            Key::Backspace => {
                delete_last_char(&mut _stdout, cursor_x, cursor_y, min_cursor_x_bound);

                char_buf.pop();

                if search_mode {
                    // already typed command
                    let command: String = char_buf.iter().collect();

                    // get suggestions for this command
                    suggestions = history_handler.search(&command);

                    suggestion_index = 0;

                    print_suggestion(
                        &command,
                        suggestions.get(suggestion_index),
                        cursor_y,
                        &mut _stdout,
                    );

                    min_cursor_x_bound = 5;
                }
            }
            _ => continue,
        }

        // Flush again.
        _stdout.flush().unwrap();
    }
    // go to a clear line and exit
    print!("\n\r");
    _stdout.flush().unwrap();
}

fn print_suggestion(
    command: &String,
    suggestion: Option<&String>,
    cursor_y: u16,
    _stdout: &mut termion::raw::RawTerminal<std::io::Stdout>,
) {
    let cropped_suggestion_text;
    if let Some(suggestion_str) = suggestion {
        cropped_suggestion_text = &suggestion_str[command.len()..];
    } else {
        cropped_suggestion_text = "";
    }

    let formatted_suggestion_text = format!(
        "{}{}{}",
        style::Underline,
        cropped_suggestion_text,
        style::NoUnderline
    );

    // print search icon
    write!(
        _stdout,
        "\r{clear_line}🔍: {command}{suggestion}{reset_cursor}",
        clear_line = termion::clear::CurrentLine,
        suggestion = formatted_suggestion_text,
        command = command,
        reset_cursor = termion::cursor::Goto(5 + command.len() as u16, cursor_y)
    )
    .unwrap();
    _stdout.flush().unwrap();
}

fn get_entries_of_glob(path_str: &str) -> glob::Paths {
    let postfix: String;

    // If user already used '*', don't add it
    if path_str.contains('*') {
        postfix = String::new();
    } else {
        postfix = String::from("*");
    }

    let path = std::path::Path::new(path_str);

    if path.is_relative() {
        glob(&format!("./{}{postfix}", path_str, postfix = postfix))
            .expect("Failed to read glob pattern")
    } else {
        glob(&format!("{}{postfix}", path_str, postfix = postfix))
            .expect("Failed to read glob pattern")
    }
}

// Prints the prompt
// Returns minimum x coordinate that cursor can get without interfering with the prompt text
fn print_prompt(stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) -> u16 {
    // Get the prompt and its size without any style and color
    let (prompt_text, prompt_len) = prompt();

    // Print the prompt
    print!("{}", prompt_text);
    stdout.flush().unwrap();

    // Cursor must be on position after the last char of prompt
    // That plus one is because coordinate starts at 1. so the 1 is offset
    (prompt_len + 1) as u16
}

// Moves the cursor one position to left until it reaches the minimum allowed value
fn move_cursor_left(
    stdout: &mut termion::raw::RawTerminal<std::io::Stdout>,
    cursor_x: u16,
    cursor_y: u16,
    min_cursor_x_bound: u16,
) {
    write!(
        stdout,
        "{}",
        termion::cursor::Goto(std::cmp::max(cursor_x - 1, min_cursor_x_bound), cursor_y),
    )
    .unwrap();
}

// Moves the cursor one position to right until it reaches the maximum allowed value
// Maximum value is the total number of typed chars
// Cursor can not go beyond last typed char
fn move_cursor_right(
    stdout: &mut termion::raw::RawTerminal<std::io::Stdout>,
    cursor_x: u16,
    cursor_y: u16,
    max_cursor_x_bound: u16,
) {
    write!(
        stdout,
        "{}",
        termion::cursor::Goto(std::cmp::min(cursor_x + 1, max_cursor_x_bound), cursor_y),
    )
    .unwrap();
}

// Replaces the character behind the cursor with space(" ")
fn delete_last_char(
    stdout: &mut termion::raw::RawTerminal<std::io::Stdout>,
    cursor_x: u16,
    cursor_y: u16,
    min_cursor_x_bound: u16,
) {
    // Must not delete any char from prompt
    if cursor_x - 1 >= min_cursor_x_bound {
        write!(
            stdout,
            "{}{}{}",
            termion::cursor::Goto(cursor_x - 1, cursor_y),
            " ",
            termion::cursor::Goto(cursor_x - 1, cursor_y),
        )
        .unwrap();
    }
}
