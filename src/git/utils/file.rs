use std::path::Path;

use crate::git::{error::GitError, result::MyResult};

/// Validate if the path exist and if it is a file
pub fn validate_file(file: &Path) -> MyResult<()> {
    if !file.exists() {
        return Err(GitError::general(format!(
            "File: {} does not exists.",
            file.display()
        )));
    }

    if !file.is_file() {
        return Err(GitError::general(format!(
            "File: {} must be a file.",
            file.display()
        )));
    }

    Ok(())
}
