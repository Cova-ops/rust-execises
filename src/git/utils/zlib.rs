use flate2::{
    Compression,
    bufread::{ZlibDecoder, ZlibEncoder},
};
use std::io::Read;

use crate::git::result::MyResult;

pub fn compress_content(content: &[u8]) -> MyResult<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(content, Compression::default());
    let mut compressed = Vec::new();

    encoder.read_to_end(&mut compressed)?;

    Ok(compressed)
}

pub fn decompress_content(content: &[u8]) -> MyResult<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(content);
    let mut decompressed_data = Vec::new();

    decoder.read_to_end(&mut decompressed_data)?;

    Ok(decompressed_data)
}
