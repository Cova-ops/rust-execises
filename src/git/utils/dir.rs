use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use crate::git::{
    constant::{DEFAULT_CONFIG_FILE, DEFAULT_DESCRIPTION_FILE, DEFAULT_HEAD_FILE, ROOT_DIR},
    error::GitError,
    result::MyResult,
    utils::{
        sha1::{bytes_to_hex, get_sha1},
        zlib::{compress_content, decompress_content},
    },
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GitDir {
    pub git_path: PathBuf,
}

pub struct ChildEntryFiles {
    pub mode: String,
    pub name: String,
    pub hex: String,
    pub hash: Vec<u8>,
}

pub trait ChildEntry {
    fn get_content(&'_ self) -> Cow<'_, str>;
}

impl ChildEntry for Vec<ChildEntryFiles> {
    fn get_content(&'_ self) -> Cow<'_, str> {
        let mut vec_out = vec![];
        for c in self {
            vec_out.push(format!("{} {} {}", c.mode, c.name, c.hex));
        }

        Cow::Owned(vec_out.join("\n"))
    }
}

// For file's content
impl ChildEntry for String {
    fn get_content(&'_ self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

pub struct ObjectMetadata {
    pub content: Box<dyn ChildEntry>,
    pub size: String,
    pub type_object: String,
}

impl GitDir {
    pub fn new(mut root: PathBuf) -> MyResult<Self> {
        if !root.exists() {
            fs::create_dir_all(&root)?;
        }
        root.push(ROOT_DIR);

        if root.exists() {
            return Err(GitError::general(format!(
                ".git already exist in this path {}",
                root.display()
            )));
        }

        fs::create_dir(&root)?;

        // files by default
        fs::write(root.join("HEAD"), DEFAULT_HEAD_FILE)?;
        fs::write(root.join("config"), DEFAULT_CONFIG_FILE)?;
        fs::write(root.join("description"), DEFAULT_DESCRIPTION_FILE)?;

        // directories by default
        fs::create_dir(root.join("hooks"))?;
        fs::create_dir(root.join("info"))?;

        fs::create_dir_all(root.join("objects/info"))?;
        fs::create_dir(root.join("objects/pack"))?;

        fs::create_dir_all(root.join("refs/heads"))?;
        fs::create_dir(root.join("refs/tags"))?;

        Ok(Self { git_path: root })
    }

    pub fn add_object(&mut self, file: &Path) -> MyResult<Vec<u8>> {
        if !file.exists() {
            return Err(GitError::io(
                "Cannot save a object from a non existant file",
            ));
        }

        let mut content: Vec<u8> = fs::read(&file)?;
        let mut header = format!("blob {}\0", content.len()).into_bytes();
        header.extend_from_slice(&mut content);

        let hash = get_sha1(&header)?;
        let hex = bytes_to_hex(&hash);
        let compressed_content = compress_content(&header)?;

        let name_dir = hex.get(..2).unwrap();
        let name_file = hex.get(2..).unwrap();

        let path_dir = self.git_path.join("objects").join(name_dir);
        if !path_dir.exists() {
            fs::create_dir(&path_dir)?;
        }

        let path_file = path_dir.join(name_file);
        if !path_file.exists() {
            fs::write(&path_file, compressed_content)?;
        }

        Ok(hash)
    }

    pub fn add_object_tree(&mut self, content: &[u8]) -> MyResult<Vec<u8>> {
        let hash = get_sha1(content)?;
        let hex = bytes_to_hex(&hash);
        let compressed_content = compress_content(content)?;

        let name_dir = hex.get(..2).unwrap();
        let name_file = hex.get(2..).unwrap();

        let path_dir = self.git_path.join("objects").join(name_dir);

        if !path_dir.exists() {
            fs::create_dir(&path_dir)?;
        }

        let path_file = path_dir.join(name_file);
        if !path_file.exists() {
            fs::write(&path_file, compressed_content)?;
        }

        Ok(hash)
    }

    fn read_content_tree(mut content: &[u8]) -> MyResult<Vec<ChildEntryFiles>> {
        let mut vec_content = vec![];

        while !content.is_empty() {
            let idx_end_mode = content
                .iter()
                .position(|&c| c == b' ')
                .ok_or_else(|| GitError::io("File malformed"))?;

            let idx_end_name = content
                .iter()
                .position(|&c| c == b'\0')
                .ok_or_else(|| GitError::io("File malformed"))?;

            let mode = String::from_utf8(content[..idx_end_mode].to_vec())?;
            let name = String::from_utf8(content[(idx_end_mode + 1)..idx_end_name].to_vec())?;
            let hash = content[(idx_end_name + 1)..(idx_end_name + 21)].to_vec();
            let hex = bytes_to_hex(&hash);

            content = &content[idx_end_name + 21..];

            vec_content.push(ChildEntryFiles {
                name,
                mode,
                hex,
                hash,
            });
        }

        Ok(vec_content)
    }

    fn read_object_metadata(data: Vec<u8>) -> MyResult<ObjectMetadata> {
        let idx_end_type = data
            .iter()
            .position(|&c| c == b' ')
            .ok_or_else(|| GitError::io("File malformed"))?;

        let idx_end_size = data
            .iter()
            .position(|&c| c == b'\0')
            .ok_or_else(|| GitError::io("File malformed"))?;

        let type_object = String::from_utf8(data[..idx_end_type].to_vec())?;
        let size = String::from_utf8(data[(idx_end_type + 1)..idx_end_size].to_vec())?;

        let content: Box<dyn ChildEntry> = match type_object.as_str() {
            "tree" => Box::new(Self::read_content_tree(&data[idx_end_size + 1..])?),
            _ => Box::new(String::from_utf8(data[idx_end_size + 1..].to_vec())?),
        };

        Ok(ObjectMetadata {
            content,
            size,
            type_object,
        })
    }

    pub fn read_object_hash(&self, hash: &str) -> MyResult<ObjectMetadata> {
        let objects_path = self.git_path.join("objects");
        let (hash_parent, rest_hash) = hash.split_at(2);

        let dir_hash = objects_path.join(&hash_parent);
        if !dir_hash.exists() {
            return Err(GitError::io("Object hash does not exists."));
        }

        let read_dir = fs::read_dir(&dir_hash)?;

        let mut files = vec![];
        for entry in read_dir {
            let path = entry?.path();

            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|v| v.to_str()) {
                Some(v) => v,
                _ => continue,
            };

            if !file_name.starts_with(rest_hash) {
                continue;
            }

            files.push(file_name.to_string());
        }

        if files.len() == 0 {
            return Err(GitError::io("Object hash does not exists."));
        }

        if files.len() > 1 {
            return Err(GitError::io(
                "More than 1 file with taht hash, add more digest.",
            ));
        }

        // SAFETY: We are sure there is only 1 element in the vec
        let dir_file = dir_hash.join(files.get(0).unwrap());

        let raw_content = fs::read(dir_file)?;
        let decompress_content = decompress_content(&raw_content)?;
        let metadata = Self::read_object_metadata(decompress_content)?;

        Ok(metadata)
    }
}

impl TryFrom<PathBuf> for GitDir {
    type Error = GitError;

    fn try_from(mut path: PathBuf) -> MyResult<Self> {
        if !path.exists() {
            return Err(GitError::General(format!(
                "Path: {} does not exist.",
                path.display()
            )));
        }
        path.push(ROOT_DIR);

        if !path.exists() {
            return Err(GitError::general(
                "This project has not been initiated. Try: git init",
            ));
        }

        let files = &[
            "HEAD",
            "config",
            "description",
            "hooks",
            "info",
            "objects/info",
            "objects/pack",
            "refs/heads",
            "refs/tags",
        ];
        for file in files {
            if !path.join(file).exists() {
                return Err(GitError::general(
                    "This project has not been initiated. Try: git init",
                ));
            }
        }

        Ok(Self { git_path: path })
    }
}

#[cfg(test)]
mod test_init {
    use super::*;
    use tempdir::{self, TempDir};

    #[test]
    fn dir_already_created() {
        let tmp_dir = TempDir::new("example").unwrap();

        let root_path = tmp_dir.into_path();
        let git_path = root_path.join(ROOT_DIR);

        fs::create_dir_all(git_path.as_path()).unwrap();

        let result = GitDir::new(root_path).unwrap_err();

        assert_eq!(
            result,
            GitError::general(format!(
                ".git already exist in this path {}",
                git_path.display()
            ))
        );
    }

    #[test]
    fn successfull_creation() {
        let tmp_dir = TempDir::new("example").unwrap();

        let root_path = tmp_dir.into_path();
        let git_path = root_path.join(ROOT_DIR);

        GitDir::new(root_path.to_owned()).unwrap();

        // files by default
        assert!(git_path.join("HEAD").exists());
        assert!(git_path.join("config").exists());
        assert!(git_path.join("description").exists());

        assert_eq!(
            fs::read_to_string(git_path.join("HEAD")).unwrap(),
            DEFAULT_HEAD_FILE
        );
        assert_eq!(
            fs::read_to_string(git_path.join("description")).unwrap(),
            DEFAULT_DESCRIPTION_FILE
        );
        assert_eq!(
            fs::read_to_string(git_path.join("config")).unwrap(),
            DEFAULT_CONFIG_FILE
        );

        // directories by default
        assert!(git_path.join("hooks").exists());
        assert!(git_path.join("info").exists());
        assert!(git_path.join("objects/info").exists());
        assert!(git_path.join("objects/pack").exists());
        assert!(git_path.join("refs/heads").exists());
        assert!(git_path.join("refs/tags").exists());
    }

    mod test_add_object {
        use super::*;

        #[test]
        fn files_does_exist() -> MyResult<()> {
            let dir = TempDir::new("")?;
            let path = dir.into_path();
            let imaginary_file = path.join("am_not_a_file.txt");

            let mut dir = GitDir::new(path)?;
            let error = dir.add_object(&imaginary_file).unwrap_err();

            assert_eq!(
                error,
                GitError::io("Cannot save a object from a non existant file")
            );

            Ok(())
        }

        #[test]
        fn empty_file() -> MyResult<()> {
            let dir = TempDir::new("")?;
            let path = dir.into_path();
            let file = path.join("empty_file.txt");

            fs::write(&file, "")?;

            let mut dir = GitDir::new(path.clone())?;
            let hash = dir.add_object(&file).unwrap();
            let hex = bytes_to_hex(&hash);

            // "" to hex -> e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
            assert_eq!(hex, "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");

            let mut path_final = path.as_path().join(".git/objects");

            path_final.push("e6");
            assert!(fs::exists(&path_final)?);

            path_final.push("9de29bb2d1d6434b8b29ae775ad8c2e48c5391");
            assert!(fs::exists(path_final)?);

            Ok(())
        }

        #[test]
        fn file_is_created() -> MyResult<()> {
            let dir = TempDir::new("")?;
            let path = dir.into_path();
            let file = path.join("am_a_file.txt");

            fs::write(&file, "Hello world!")?;

            let mut dir = GitDir::new(path.clone())?;
            let hash = dir.add_object(&file)?;
            let digest = bytes_to_hex(&hash);

            // "Hello world!" to hex -> 6769dd60bdf536a83c9353272157893043e9f7d0
            assert_eq!(digest, "6769dd60bdf536a83c9353272157893043e9f7d0");

            let mut path_final = path.as_path().join(".git/objects");

            path_final.push("67");
            assert!(fs::exists(&path_final)?);

            path_final.push("69dd60bdf536a83c9353272157893043e9f7d0");
            assert!(fs::exists(path_final)?);

            Ok(())
        }
    }
}
