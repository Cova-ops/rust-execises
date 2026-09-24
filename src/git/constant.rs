pub static ROOT_DIR: &str = ".git";

// Default values for init command
pub static DEFAULT_HEAD_FILE: &str = "ref: refs/heads/main\n";
pub static DEFAULT_DESCRIPTION_FILE: &str =
    "Unnamed repository; edit this file 'description' to name the repository.";
pub static DEFAULT_CONFIG_FILE: &str = r#"[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true"#;
