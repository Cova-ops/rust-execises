use crate::git::error::GitError;

pub type MyResult<T> = Result<T, GitError>;
