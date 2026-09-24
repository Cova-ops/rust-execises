use std::path::PathBuf;

use crate::git::{result::MyResult, utils::dir::GitDir};

pub fn run(path: Option<PathBuf>) -> MyResult<()> {
    let path = match path {
        None => std::env::current_dir()?,
        Some(v) => v,
    };

    GitDir::new(path)?;

    Ok(())
}
