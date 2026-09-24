use sha1::{Digest, Sha1};

use crate::git::result::MyResult;

pub fn get_sha1(content: &[u8]) -> MyResult<Vec<u8>> {
    let mut hasher = Sha1::new();
    hasher.update(content);
    let hash = hasher.finalize();

    Ok(hash.to_vec())
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}
