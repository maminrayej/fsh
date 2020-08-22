mod history;
mod preprocess;
mod process;
mod read;

use history::get_history_handler;

fn main() {
    ctrlc::set_handler(move || {}).expect("Error setting Ctrl-C handler");

    let history_handler = get_history_handler();
    read::read_loop(history_handler);
}
