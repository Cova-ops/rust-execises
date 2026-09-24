use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

mod test_e2e_git {
    use super::*;

    fn executable() -> String {
        std::env::var("CARGO_BIN_EXE_rust-execises")
            .expect("CARGO_BIN_EXE_rust-execises is not defined")
    }

    fn run<I, S>(args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new(executable())
            .args(args)
            .output()
            .expect("failed to execute rust-execises")
    }

    fn git<I, S>(repo: &Path, args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .expect("failed to execute git")
    }

    fn stdout(output: &Output) -> String {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn stderr(output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    }

    fn assert_success(output: &Output) {
        assert!(
            output.status.success(),
            "command failed\nstdout:\n{}\nstderr:\n{}",
            stdout(output),
            stderr(output),
        );
    }

    fn temp_repo(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("invalid system time")
            .as_nanos();

        std::env::temp_dir().join(format!("mein-git-{name}-{timestamp}"))
    }

    fn init_repo(name: &str) -> PathBuf {
        let repo = temp_repo(name);

        let output = run(["git", "init", repo.to_str().unwrap()]);

        assert_success(&output);

        repo
    }

    fn write_tree(repo: &Path) -> String {
        let output = run(["git", "write-tree", "-C", repo.to_str().unwrap()]);

        assert_success(&output);

        stdout(&output)
    }

    mod init {
        use super::*;

        #[test]
        fn creates_valid_git_repository() {
            let repo = temp_repo("init");

            let output = run(["git", "init", repo.to_str().unwrap()]);

            assert_success(&output);

            assert!(repo.join(".git").exists());
            assert!(repo.join(".git/objects").exists());
            assert!(repo.join(".git/refs/heads").exists());
            assert!(repo.join(".git/refs/tags").exists());

            let head = fs::read_to_string(repo.join(".git/HEAD")).unwrap();

            assert_eq!(head, "ref: refs/heads/main\n");

            let git_status = git(&repo, ["status"]);

            assert_success(&git_status);

            fs::remove_dir_all(repo).unwrap();
        }
    }

    mod hash_object {
        use super::*;

        #[test]
        fn creates_git_compatible_blob() {
            let repo = init_repo("hash-object");

            let file = repo.join("hello.txt");

            fs::write(&file, "Hello World!\n").unwrap();

            let output = run([
                "git",
                "hash-object",
                "-w",
                file.to_str().unwrap(),
                "-C",
                repo.to_str().unwrap(),
            ]);

            assert_success(&output);

            let my_hash = stdout(&output);

            let git_output = git(&repo, ["hash-object", file.to_str().unwrap()]);

            assert_success(&git_output);

            assert_eq!(my_hash, stdout(&git_output));

            let object_path = repo
                .join(".git/objects")
                .join(&my_hash[..2])
                .join(&my_hash[2..]);

            assert!(object_path.exists(), "object was not written");

            fs::remove_dir_all(repo).unwrap();
        }
    }

    mod cat_file {
        use super::*;

        fn create_blob(repo: &Path) -> String {
            let file = repo.join("hello.txt");

            fs::write(&file, "Hello World!\n").unwrap();

            let output = run([
                "git",
                "hash-object",
                "-w",
                file.to_str().unwrap(),
                "-C",
                repo.to_str().unwrap(),
            ]);

            assert_success(&output);

            stdout(&output)
        }

        #[test]
        fn pretty_print_matches_git() {
            let repo = init_repo("cat-file-p");
            let hash = create_blob(&repo);

            let my_output = run(["git", "cat-file", "-p", &hash, "-C", repo.to_str().unwrap()]);

            let git_output = git(&repo, ["cat-file", "-p", &hash]);

            assert_success(&my_output);
            assert_success(&git_output);

            assert_eq!(stdout(&my_output), stdout(&git_output));

            fs::remove_dir_all(repo).unwrap();
        }

        #[test]
        fn type_matches_git() {
            let repo = init_repo("cat-file-t");
            let hash = create_blob(&repo);

            let my_output = run(["git", "cat-file", "-t", &hash, "-C", repo.to_str().unwrap()]);

            let git_output = git(&repo, ["cat-file", "-t", &hash]);

            assert_success(&my_output);
            assert_success(&git_output);

            assert_eq!(stdout(&my_output), stdout(&git_output));

            fs::remove_dir_all(repo).unwrap();
        }

        #[test]
        fn size_matches_git() {
            let repo = init_repo("cat-file-s");
            let hash = create_blob(&repo);

            let my_output = run(["git", "cat-file", "-s", &hash, "-C", repo.to_str().unwrap()]);

            let git_output = git(&repo, ["cat-file", "-s", &hash]);

            assert_success(&my_output);
            assert_success(&git_output);

            assert_eq!(stdout(&my_output), stdout(&git_output));

            fs::remove_dir_all(repo).unwrap();
        }

        #[test]
        fn exists_returns_correct_status() {
            let repo = init_repo("cat-file-e");
            let hash = create_blob(&repo);

            let existing = run(["git", "cat-file", "-e", &hash, "-C", repo.to_str().unwrap()]);

            assert!(existing.status.success());

            let missing = run([
                "git",
                "cat-file",
                "-e",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "-C",
                repo.to_str().unwrap(),
            ]);

            assert!(!missing.status.success());

            fs::remove_dir_all(repo).unwrap();
        }
    }

    mod write_tree {
        use super::*;

        fn populate_repo(repo: &Path) {
            fs::create_dir_all(repo.join("src/utils")).unwrap();

            fs::create_dir_all(repo.join("docs")).unwrap();

            fs::write(repo.join("README.md"), "Hello World!\n").unwrap();

            fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();

            fs::write(repo.join("src/utils/helper.rs"), "pub fn helper() {}\n").unwrap();

            fs::write(repo.join("docs/README.md"), "Documentation\n").unwrap();

            fs::write(repo.join("empty.txt"), "").unwrap();

            let script = repo.join("run.sh");

            fs::write(&script, "#!/bin/sh\necho hello\n").unwrap();

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                let mut permissions = fs::metadata(&script).unwrap().permissions();

                permissions.set_mode(0o755);

                fs::set_permissions(script, permissions).unwrap();
            }
        }

        #[test]
        fn creates_git_compatible_tree() {
            let repo = init_repo("write-tree");
            populate_repo(&repo);

            let tree = write_tree(&repo);

            let output = git(&repo, ["cat-file", "-t", &tree]);

            assert_success(&output);

            assert_eq!(stdout(&output), "tree");

            let output = git(&repo, ["cat-file", "-p", &tree]);

            assert_success(&output);

            let content = stdout(&output);

            assert!(content.contains("README.md"));

            assert!(content.contains("src"));

            assert!(content.contains("docs"));

            assert!(content.contains("empty.txt"));

            fs::remove_dir_all(repo).unwrap();
        }

        #[test]
        fn preserves_git_modes() {
            let repo = init_repo("write-tree-modes");

            populate_repo(&repo);

            let tree = write_tree(&repo);

            let output = git(&repo, ["cat-file", "-p", &tree]);

            assert_success(&output);

            let content = stdout(&output);

            assert!(
                content
                    .lines()
                    .any(|line| { line.starts_with("100644 blob") && line.ends_with("README.md") })
            );

            assert!(
                content
                    .lines()
                    .any(|line| { line.starts_with("040000 tree") && line.ends_with("src") })
            );

            #[cfg(unix)]
            assert!(
                content
                    .lines()
                    .any(|line| { line.starts_with("100755 blob") && line.ends_with("run.sh") })
            );

            fs::remove_dir_all(repo).unwrap();
        }

        #[test]
        fn is_deterministic() {
            let repo = init_repo("write-tree-deterministic");

            populate_repo(&repo);

            let first = write_tree(&repo);
            let second = write_tree(&repo);

            assert_eq!(first, second, "same files should produce same tree");

            fs::remove_dir_all(repo).unwrap();
        }

        #[test]
        fn nested_change_changes_root_tree() {
            let repo = init_repo("write-tree-change");

            populate_repo(&repo);

            let before = write_tree(&repo);

            fs::write(
                repo.join("src/utils/helper.rs"),
                "pub fn helper() { println!(\"changed\"); }\n",
            )
            .unwrap();

            let after = write_tree(&repo);

            assert_ne!(
                before, after,
                "nested file change should propagate to root tree"
            );

            fs::remove_dir_all(repo).unwrap();
        }

        #[test]
        fn objects_pass_git_fsck() {
            let repo = init_repo("write-tree-fsck");

            populate_repo(&repo);

            write_tree(&repo);

            let output = git(&repo, ["fsck", "--full"]);

            let result = format!("{}\n{}", stdout(&output), stderr(&output));

            assert!(!result.contains("corrupt"), "{result}");

            assert!(!result.contains("fatal:"), "{result}");

            fs::remove_dir_all(repo).unwrap();
        }
    }

    mod commit_tree {
        use super::*;

        fn create_tree(repo: &Path) -> String {
            fs::write(repo.join("README.md"), "Initial version\n").unwrap();

            write_tree(repo)
        }

        #[test]
        fn creates_git_compatible_commit() {
            let repo = init_repo("commit-tree");

            let tree = create_tree(&repo);

            let output = run([
                "git",
                "commit-tree",
                &tree,
                "-m",
                "Initial commit",
                "-C",
                repo.to_str().unwrap(),
            ]);

            assert_success(&output);

            let commit = stdout(&output);

            let output = git(&repo, ["cat-file", "-t", &commit]);

            assert_success(&output);

            assert_eq!(stdout(&output), "commit");

            let output = git(&repo, ["cat-file", "-p", &commit]);

            assert_success(&output);

            let content = stdout(&output);

            assert!(content.contains(&format!("tree {tree}")));

            assert!(content.contains("Initial commit"));

            fs::remove_dir_all(repo).unwrap();
        }

        #[test]
        fn creates_commit_with_parent() {
            let repo = init_repo("commit-tree-parent");

            let first_tree = create_tree(&repo);

            let first = run([
                "git",
                "commit-tree",
                &first_tree,
                "-m",
                "Initial commit",
                "-C",
                repo.to_str().unwrap(),
            ]);

            assert_success(&first);

            let first_commit = stdout(&first);

            fs::write(repo.join("README.md"), "Second version\n").unwrap();

            let second_tree = write_tree(&repo);

            let second = run([
                "git",
                "commit-tree",
                &second_tree,
                "-p",
                &first_commit,
                "-m",
                "Second commit",
                "-C",
                repo.to_str().unwrap(),
            ]);

            assert_success(&second);

            let second_commit = stdout(&second);

            let output = git(&repo, ["cat-file", "-p", &second_commit]);

            assert_success(&output);

            let content = stdout(&output);

            assert!(content.contains(&format!("parent {first_commit}")));

            assert!(content.contains(&format!("tree {second_tree}")));

            assert!(content.contains("Second commit"));

            fs::remove_dir_all(repo).unwrap();
        }
    }
}
