use chrono::Local;

use crate::git::{args::CommitTreeArgs, error::GitError, result::MyResult};

/// The output should be:
/// ```
/// tree <hash>\n
/// parent <commit-hash>\n
/// author <name> <email> <timestamp> <timezone>\n
/// committer <name> <email> <timestamp> <timezone>\n
/// \n
/// <commit message>\n
/// ```
fn create_payload(args: &CommitTreeArgs) -> MyResult<String> {
    let tree = match args.dir.read_object_hash(&args.tree_hash) {
        Ok(v) if v.type_object == "tree" => format!("tree {}", &args.tree_hash),
        Err(v) => {
            return Err(v);
        }
        _ => {
            return Err(GitError::args(
                "commit-tree",
                "Tree hash need to be a valid one",
            ));
        }
    };

    let parent_hash: String = match args.parent_hash.as_deref() {
        None => String::new(),
        Some(hash) => match args.dir.read_object_hash(hash) {
            Ok(v) if v.type_object == "commit" => format!("parent {hash}\n"),
            _ => {
                return Err(GitError::args(
                    "commit-tree",
                    "Parent hash need to be a valid one",
                ));
            }
        },
    };

    let now = Local::now().offset().local_minus_utc();
    let time_zone_offset = Local::now().offset().to_string().replace(":", "");
    let now_format = format!("{now} {time_zone_offset}");

    // For testing, author and committer
    let author = format!("Juan <juan@example.com> {now_format}");
    let committer = format!("Juan <juan@example.com> {now_format}");

    let payload = format!(
        r#"{tree}
{parent_hash}{author}
{committer}

{}\n"#,
        args.commit
    );

    Ok(payload)
}

pub fn run(mut args: CommitTreeArgs) -> MyResult<()> {
    let payload = create_payload(&args)?;

    let content = format!("commit {}\0{payload}", payload.len());

    let hash = args.dir.add_object_tree(content.as_bytes())?;
    let hex: String = hash.into_iter().map(|v| format!("{v:02x}")).collect();

    println!("{hex}");

    Ok(())
}
