use std::fs::{self};

use crate::git::{
    args::HashObjectArgs,
    result::MyResult,
    utils::{
        file::validate_file,
        sha1::{bytes_to_hex, get_sha1},
    },
};

pub fn run(mut props: HashObjectArgs) -> MyResult<()> {
    validate_file(props.path.as_ref())?;

    if props.write {
        let hash = props.dir.add_object(&props.path)?;
        let digest = bytes_to_hex(&hash);

        println!("{digest}");

        return Ok(());
    }

    let mut content = fs::read(&props.path)?;

    let mut header = format!("blob {}\0", content.len()).into_bytes();
    header.extend_from_slice(&mut content);

    let digest = get_sha1(&header)?;
    let digest = bytes_to_hex(&digest);

    println!("{digest}");

    Ok(())
}
