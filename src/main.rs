mod history;
mod preprocess;
mod process;
mod read;

use history::get_history_handler;

fn main() {
    // Ignore Ctrl+C signal
    ctrlc::set_handler(move || {}).expect("Error setting Ctrl-C handler");

    // Get a history handler
    let history_handler = get_history_handler();

    // Start shell
    read::read_loop(history_handler);
}
