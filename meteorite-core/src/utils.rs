/*
    meteorite  Fast, Secure & Easy-to-use Matrix client in Rust
    Copyright (C) 2026  Paul8711

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as
    published by the Free Software Foundation, either version 3 of the
    License, or (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
