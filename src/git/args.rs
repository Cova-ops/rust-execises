use std::path::PathBuf;

use crate::git::{error::GitError, result::MyResult, utils::dir::GitDir};

#[derive(Debug, PartialEq, Eq)]
pub struct HashObjectArgs {
    pub path: PathBuf,
    pub write: bool,
    pub dir: GitDir,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CatFileActions {
    ObjectContent, // -p
    ObjectType,    // -t
    ObjectSize,    // -s
    ObjectExist,   // -e
}

#[derive(Debug, PartialEq, Eq)]
pub struct CatFileArgs {
    pub dir: GitDir,
    pub action: CatFileActions,
    pub hash: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommitTreeArgs {
    pub parent_hash: Option<String>,
    pub dir: GitDir,
    pub commit: String,
    pub tree_hash: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Init(Option<PathBuf>),
    HashObject(HashObjectArgs),
    CatFile(CatFileArgs),
    WriteTree(GitDir),
    CommitTree(CommitTreeArgs),
}

fn guard_init_args(args: &[String]) -> MyResult<Command> {
    let path = args.get(0).map(|v| PathBuf::from(v));
    Ok(Command::Init(path))
}

fn guard_hash_object_args(args: &[String]) -> MyResult<Command> {
    let mut path: Option<PathBuf> = None;
    let mut write: bool = false;
    let mut dir: PathBuf = std::env::current_dir()?;

    let mut omit_next = false;
    for i in 0..args.len() {
        if omit_next {
            omit_next = false;
            continue;
        }

        let arg = args.get(i).unwrap();

        if arg.starts_with("-") {
            match arg.as_ref() {
                "-w" => write = true,
                "-C" => {
                    dir = {
                        omit_next = true;

                        match args.get(i + 1) {
                            Some(v) => PathBuf::from(v),
                            _ => {
                                return Err(GitError::args(
                                    "hash-object",
                                    "After -C must be a valid path",
                                ));
                            }
                        }
                    }
                }
                _ => return Err(GitError::args("hash-object", "Flag {arg} is not supported")),
            }

            continue;
        }

        if path.is_none() {
            path = Some(PathBuf::from(arg));
        }
    }

    let path = path.ok_or_else(|| GitError::args("hash-object", "Missing a file path"))?;
    let dir = GitDir::try_from(dir)?;

    Ok(Command::HashObject(HashObjectArgs { path, write, dir }))
}

fn guard_cat_file_args(args: &[String]) -> MyResult<Command> {
    let mut dir: PathBuf = std::env::current_dir()?;
    let mut action: Option<CatFileActions> = None;
    let mut hash: Option<String> = None;

    let mut omit_next = false;
    for i in 0..args.len() {
        if omit_next {
            omit_next = false;
            continue;
        }

        // SAFETY: Inside of the for
        let arg = args.get(i).unwrap();

        if arg.starts_with("-") {
            match arg.as_ref() {
                "-p" => {
                    omit_next = true;

                    if action.is_some() {
                        return Err(GitError::args("cat-file", "Only one action per command"));
                    }

                    action = Some(CatFileActions::ObjectContent);
                    hash = match args.get(i + 1) {
                        Some(v) => Some(v.to_string()),
                        _ => {
                            return Err(GitError::args(
                                "cat-file",
                                "After -p must be a valid hash",
                            ));
                        }
                    }
                }
                "-t" => {
                    omit_next = true;

                    if action.is_some() {
                        return Err(GitError::args("cat-file", "Only one action per command"));
                    }

                    action = Some(CatFileActions::ObjectType);
                    hash = match args.get(i + 1) {
                        Some(v) => Some(v.to_string()),
                        _ => {
                            return Err(GitError::args(
                                "cat-file",
                                "After -t must be a valid hash",
                            ));
                        }
                    }
                }
                "-s" => {
                    omit_next = true;

                    if action.is_some() {
                        return Err(GitError::args("cat-file", "Only one action per command"));
                    }

                    action = Some(CatFileActions::ObjectSize);
                    hash = match args.get(i + 1) {
                        Some(v) => Some(v.to_string()),
                        _ => {
                            return Err(GitError::args(
                                "cat-file",
                                "After -s must be a valid hash",
                            ));
                        }
                    }
                }
                "-e" => {
                    omit_next = true;

                    if action.is_some() {
                        return Err(GitError::args("cat-file", "Only one action per command"));
                    }

                    action = Some(CatFileActions::ObjectExist);
                    hash = match args.get(i + 1) {
                        Some(v) => Some(v.to_string()),
                        _ => {
                            return Err(GitError::args(
                                "cat-file",
                                "After -e must be a valid hash",
                            ));
                        }
                    }
                }
                "-C" => {
                    dir = {
                        omit_next = true;

                        match args.get(i + 1) {
                            Some(v) => PathBuf::from(v),
                            _ => {
                                return Err(GitError::args(
                                    "cat-file",
                                    "After -C must be a valid path",
                                ));
                            }
                        }
                    }
                }
                _ => return Err(GitError::args("cat-file", "Flag {arg} is not supported")),
            }

            continue;
        }
    }

    let action =
        action.ok_or_else(|| GitError::args("cat-file", "Missing action (-p, -t, -s, -e)"))?;
    let hash = hash.ok_or_else(|| GitError::args("cat-file", "Missing hash"))?;
    let dir = GitDir::try_from(dir)?;

    Ok(Command::CatFile(CatFileArgs { action, dir, hash }))
}

fn guard_write_tree_args(args: &[String]) -> MyResult<Command> {
    let mut dir: PathBuf = std::env::current_dir()?;

    let mut omit_next = false;
    for i in 0..args.len() {
        if omit_next {
            omit_next = false;
            continue;
        }

        // SAFETY: Inside of the for
        let arg = args.get(i).unwrap();

        if arg.starts_with("-") {
            match arg.as_ref() {
                "-C" => {
                    omit_next = true;
                    dir = match args.get(i + 1) {
                        Some(v) => PathBuf::from(v),
                        _ => {
                            return Err(GitError::args(
                                "write-tree",
                                "After -C must be a valid path",
                            ));
                        }
                    }
                }
                _ => return Err(GitError::args("write-tree", "Flag {arg} is not supported")),
            }

            continue;
        }
    }

    let dir = GitDir::try_from(dir)?;
    Ok(Command::WriteTree(dir))
}

fn guard_commit_tree_args(args: &[String]) -> MyResult<Command> {
    let mut dir: PathBuf = std::env::current_dir()?;
    let mut parent_hash: Option<String> = None;
    let mut commit: Option<String> = None;
    let mut tree_hash: Option<String> = None;

    let mut omit_next = false;
    for i in 0..args.len() {
        if omit_next {
            omit_next = false;
            continue;
        }

        // SAFETY: Inside of the for
        let arg = args.get(i).unwrap();

        if arg.starts_with("-") {
            match arg.as_ref() {
                "-p" => {
                    omit_next = true;
                    parent_hash = match args.get(i + 1) {
                        Some(v) => Some(v.to_string()),
                        _ => {
                            return Err(GitError::args(
                                "commit-tree",
                                "After -p must be a valid hash",
                            ));
                        }
                    }
                }
                "-m" => {
                    omit_next = true;
                    commit = match args.get(i + 1) {
                        Some(v) => Some(v.to_string()),
                        _ => {
                            return Err(GitError::args(
                                "commit-tree",
                                "After -m must be a valid commit message",
                            ));
                        }
                    }
                }
                "-C" => {
                    omit_next = true;
                    dir = match args.get(i + 1) {
                        Some(v) => PathBuf::from(v),
                        _ => {
                            return Err(GitError::args(
                                "write-tree",
                                "After -C must be a valid path",
                            ));
                        }
                    }
                }
                _ => return Err(GitError::args("write-tree", "Flag {arg} is not supported")),
            }

            continue;
        }

        if tree_hash.is_none() {
            tree_hash = Some(arg.to_string());
        }
    }

    let dir = GitDir::try_from(dir)?;

    let commit = commit.ok_or_else(|| {
        GitError::args(
            "commit-tree",
            "A commit message must be defined, please use -m.",
        )
    })?;
    let tree_hash =
        tree_hash.ok_or_else(|| GitError::args("commit-tree", "A tree must be defined"))?;

    Ok(Command::CommitTree(CommitTreeArgs {
        parent_hash,
        dir,
        commit,
        tree_hash,
    }))
}

pub fn validate_args(args: &[String]) -> MyResult<Command> {
    let command = match args.get(0) {
        Some(v) if v.len() > 0 => v,
        _ => return Err(GitError::args("", "Missing command")),
    };

    // As it return an array it will never be None
    let extra_args = args.get(1..).unwrap();

    return match command.as_ref() {
        "init" => guard_init_args(extra_args),
        "hash-object" => guard_hash_object_args(extra_args),
        "cat-file" => guard_cat_file_args(extra_args),
        "write-tree" => guard_write_tree_args(extra_args),
        "commit-tree" => guard_commit_tree_args(extra_args),
        _ => Err(GitError::args(command.as_ref(), "Command not supported")),
    };
}

#[cfg(test)]
mod test {
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn invalid_args() {
        let args = &[];

        let result = validate_args(args).unwrap_err();
        assert_eq!(result, GitError::args("", "Missing command"));

        let args = &["invalid_command".into()];
        let result = validate_args(args).unwrap_err();
        assert_eq!(
            result,
            GitError::args("invalid_command", "Command not supported")
        );
    }

    #[test]
    fn validate_init_command() {
        let args = &["init".into()];
        let result = validate_args(args).unwrap();
        assert_eq!(result, Command::Init(None));

        let args = &["init".into(), "/tmp".into()];
        let result = validate_args(args).unwrap();
        assert_eq!(result, Command::Init(Some(PathBuf::from("/tmp"))));

        let args = &["init".into(), "/tmp".into(), "foo".into()];
        let result = validate_args(args).unwrap();
        assert_eq!(result, Command::Init(Some(PathBuf::from("/tmp"))));
    }

    #[test]
    fn validate_hash_object_command() {
        let helper_file = "a.txt";

        let args = &["hash-object".into()];
        let result = validate_args(args).unwrap_err();
        assert_eq!(result, GitError::args("hash-object", "Missing a file path"));

        let path_ie = TempDir::new("").unwrap();
        let path_ie = path_ie.into_path().to_str().unwrap().to_owned();

        let git_dir_current = GitDir::try_from(std::env::current_dir().unwrap()).unwrap();

        let file = format!("{path_ie}/{helper_file}");
        let args = &["hash-object".into(), file.clone()];
        let result = validate_args(args).unwrap();
        assert_eq!(
            result,
            Command::HashObject(HashObjectArgs {
                path: PathBuf::from(&file),
                write: false,
                dir: git_dir_current.clone()
            })
        );

        let args = &["hash-object".into(), file.clone(), "foo".into()];
        let result = validate_args(args).unwrap();
        assert_eq!(
            result,
            Command::HashObject(HashObjectArgs {
                path: PathBuf::from(&file),
                write: false,
                dir: git_dir_current.clone()
            })
        );

        let args = &["hash-object".into(), file.clone(), "-w".into()];
        let result = validate_args(args).unwrap();
        assert_eq!(
            result,
            Command::HashObject(HashObjectArgs {
                path: PathBuf::from(&file),
                write: true,
                dir: git_dir_current.clone()
            })
        );

        let git_dir = GitDir::new(PathBuf::from(&path_ie)).unwrap();
        let args = &[
            "hash-object".into(),
            file.clone(),
            "-w".into(),
            "-C".into(),
            path_ie.clone(),
        ];
        let result = validate_args(args).unwrap();
        assert_eq!(
            result,
            Command::HashObject(HashObjectArgs {
                path: PathBuf::from(&file),
                write: true,
                dir: git_dir.clone()
            })
        );

        let args = &["hash-object".into(), file.clone(), "-w".into(), "-C".into()];
        let result = validate_args(args).unwrap_err();
        assert_eq!(
            result,
            GitError::args("hash-object", "After -C must be a valid path")
        );
    }
}
