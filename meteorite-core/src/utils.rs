use directories::ProjectDirs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// a dirty helper function for getting the value inside a... well THAT abomination
pub(crate) fn unwrap_lock<T: Clone>(lock: &OnceLock<Mutex<T>>) -> T {
    // unwrap should be safe as long as i use this responsibly
    lock.get().unwrap().lock().unwrap().clone()
}

// parses an app id into the data dir (absolute path)
pub(crate) fn local_data_dir(app_id: &str) -> Option<PathBuf> {
    let mut parts = app_id.splitn(3, '.');

    let qualifier = parts.next()?;
    let organization = parts.next()?;
    let application = parts.next()?;

    ProjectDirs::from(qualifier, organization, application)
        .map(|dirs| dirs.data_local_dir().to_path_buf())
}
