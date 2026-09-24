use core::fmt;
use std::error::Error;

#[derive(PartialEq, Eq, Debug)]
pub struct ArgsError {
    command: String,
    message: String,
}

#[derive(PartialEq, Eq, Debug)]
pub enum GitError {
    Args(ArgsError),
    Io(String),
    General(String),
}

impl GitError {
    pub fn args<S>(command: S, message: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Args(ArgsError {
            command: command.as_ref().into(),
            message: message.as_ref().into(),
        })
    }

    pub fn io<S>(message: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Io(message.as_ref().into())
    }

    pub fn general<S>(message: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Io(message.as_ref().into())
    }
}

impl From<GitError> for String {
    fn from(value: GitError) -> Self {
        match value {
            GitError::Args(a) => {
                format!("In command {}, error: {}", a.command, a.message)
            }
            GitError::Io(s) => format!("Io error: {s}"),
            GitError::General(s) => s,
        }
    }
}

impl From<std::io::Error> for GitError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<core::str::Utf8Error> for GitError {
    fn from(value: core::str::Utf8Error) -> Self {
        Self::General(value.to_string())
    }
}

impl From<std::string::FromUtf8Error> for GitError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::General(value.to_string())
    }
}

impl From<std::time::SystemTimeError> for GitError {
    fn from(value: std::time::SystemTimeError) -> Self {
        Self::General(value.to_string())
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::Args(a) => {
                write!(f, "In command {}, error: {}", a.command, a.message)
            }
            GitError::Io(s) => write!(f, "Io error: {s}"),
            GitError::General(s) => write!(f, "{s}"),
        }
    }
}

impl Error for GitError {}
