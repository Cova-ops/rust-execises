use crate::git::{
    args::{Command, validate_args},
    commands::{cat_file, commit_tree, hash_object, init, write_tree},
};

mod args;
mod commands;
mod constant;
mod error;
mod result;
mod utils;

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let command: Command = validate_args(args)?;

    match command {
        Command::Init(p) => init::run(p)?,
        Command::HashObject(p) => hash_object::run(p)?,
        Command::CatFile(p) => cat_file::run(p)?,
        Command::WriteTree(p) => write_tree::run(p)?,
        Command::CommitTree(p) => commit_tree::run(p)?,
    }

    Ok(())
}
