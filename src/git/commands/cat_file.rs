use crate::git::{
    args::{
        CatFileActions::{self},
        CatFileArgs,
    },
    error::GitError,
    result::MyResult,
};

pub fn run(props: CatFileArgs) -> MyResult<()> {
    let metadata = match props.dir.read_object_hash(&props.hash) {
        Ok(v) => v,
        Err(_) if props.action == CatFileActions::ObjectExist => {
            return Err(GitError::general("Fatal: invalid object"));
        }
        Err(v) => return Err(v),
    };

    let result = match props.action {
        CatFileActions::ObjectType => &metadata.type_object,
        CatFileActions::ObjectSize => &metadata.size,
        CatFileActions::ObjectExist => "true",
        CatFileActions::ObjectContent => &metadata.content.get_content(),
    };

    println!("{result}");

    Ok(())
}
