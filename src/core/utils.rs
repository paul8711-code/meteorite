use directories::ProjectDirs;
use native_dialog::{DialogBuilder, MessageLevel};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// a dirty helper function for getting the value inside a... well THAT abomination
pub fn unwrap_lock(lock: &OnceLock<Mutex<PathBuf>>) -> PathBuf {
    // unwrap should be safe as long as i use this responsibly
    lock.get().unwrap().lock().unwrap().clone()
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

// parses an app id into the data dir (absolute path)
pub fn local_data_dir(app_id: &str) -> Option<PathBuf> {
    let mut parts = app_id.splitn(3, '.');

    let qualifier = parts.next()?;
    let organization = parts.next()?;
    let application = parts.next()?;

    ProjectDirs::from(qualifier, organization, application)
        .map(|dirs| dirs.data_local_dir().to_path_buf())
}
