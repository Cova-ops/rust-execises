use std::{collections::HashMap, fs, os::unix::fs::PermissionsExt, path::Path};

use crate::git::{
    result::MyResult,
    utils::{dir::GitDir, sha1::bytes_to_hex},
};

#[derive(Debug)]
struct HashEntry {
    mode: String,
    name: String,
    hash: Vec<u8>,
}

struct Tree {
    hash_entries: HashMap<String, HashEntry>,
    content: Vec<u8>,
    hash: Vec<u8>,
}

impl Tree {
    fn new() -> Self {
        Self {
            hash_entries: HashMap::new(),
            content: vec![],
            hash: vec![],
        }
    }

    fn create_content(&mut self) {
        let mut names_sorted: Vec<_> = self.hash_entries.keys().collect();

        // Sorted alphabetically
        names_sorted.sort();

        let mut size_content = 0;
        let childs: Vec<u8> = {
            let mut out = vec![];

            for key in names_sorted.into_iter() {
                // SAFETY: All name comes from hashmap key
                let v = self.hash_entries.get(key.as_str()).unwrap();

                let mut name = v.name.as_str();
                if v.mode == "40000" {
                    // Remove '/' for dirs
                    name = &name[..name.len() - 1];
                }

                let mut content = Vec::<u8>::new();
                content.extend_from_slice(v.mode.as_bytes());
                content.push(b' '); // Byte for a space
                content.extend_from_slice(name.as_bytes());
                content.push(b'\0'); // Byte for empty NUL
                content.extend_from_slice(&v.hash);

                size_content +=
                    v.mode.as_bytes().len() + 1 + name.as_bytes().len() + 1 + v.hash.len();

                // eprintln!("{} {name} {}", v.mode, bytes_to_hex(&v.hash));

                out.append(&mut content);
            }

            out
        };

        let mut header: Vec<u8> = format!("tree {size_content}\0").into_bytes();
        header.extend_from_slice(&childs);
        self.content = header
    }

    fn get_tree_recursive(root: &Path, git_dir: &mut GitDir) -> MyResult<Self> {
        let actual = fs::read_dir(&root)?;
        let mut tree = Self::new();

        for entry in actual {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Adding '/' to the dirs, it is required for sorting
                let name = match path.file_name().and_then(|v| v.to_str()) {
                    Some(v) => format!("{v}/"),
                    _ => continue,
                };

                if name == ".git/" {
                    continue;
                }

                let child_tree = Self::get_tree_recursive(&path, git_dir)?;

                let hash = child_tree.hash;
                let mode: String = "40000".into();

                tree.hash_entries
                    .insert(name.clone(), HashEntry { mode, name, hash });
            } else if path.is_file() {
                let hash = git_dir.add_object(&path)?;
                let name = match path.file_name().and_then(|v| v.to_str()) {
                    Some(v) => v.to_owned(),
                    _ => continue,
                };

                let meta = path.metadata()?;
                let is_exe = meta.permissions().mode() & 0o111 != 0;
                let mode: String = if is_exe {
                    "100755".into()
                } else {
                    "100644".into()
                };

                tree.hash_entries
                    .insert(name.clone(), HashEntry { mode, name, hash });
            } else if path.is_symlink() {
                todo!("Symlinks are not handle yet.");
            } else {
                eprintln!("File: {}", path.to_string_lossy());
                unreachable!("This file what???");
            }
        }

        tree.create_content();
        tree.hash = git_dir.add_object_tree(&tree.content)?;
        Ok(tree)
    }
}

pub fn run(mut dir: GitDir) -> MyResult<()> {
    let mut root_path = dir.git_path.clone();
    root_path.pop();

    let tree = Tree::get_tree_recursive(&root_path, &mut dir)?;
    println!("{}", bytes_to_hex(&tree.hash));

    Ok(())
}
