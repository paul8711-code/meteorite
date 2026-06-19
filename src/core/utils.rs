use native_dialog::{DialogBuilder, MessageLevel};
use std::sync::{Mutex, OnceLock};

// a dirty helper function for getting the value inside a... well THAT abomination
pub fn unwrap_lock(lock: &OnceLock<Mutex<String>>) -> String {
    // unwrap should be safe as long as i use this responsibly
    lock.get().unwrap().lock().unwrap().as_str().to_owned()
}

// helper for displaying a dialog message (usually indicating that something went wrong)
pub fn show_dialog_window(title: impl ToString, text: impl ToString, level: MessageLevel) {
    DialogBuilder::message()
        .set_title(title)
        .set_text(text)
        .set_level(level)
        .alert()
        .show()
        .unwrap();
}
