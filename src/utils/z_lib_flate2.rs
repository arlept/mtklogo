extern crate flate2;

use std::io::{Read, Result, Write};
use self::flate2::Compression;
use self::flate2::write::ZlibEncoder;
use self::flate2::read::ZlibDecoder;

// It's just a thin wrapper around 'flate2'.

pub fn inflate(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut uncompressed = Vec::new();
    decoder.read_to_end(&mut uncompressed).map(|_sz| uncompressed)
}

pub fn deflate(data: &[u8]) -> Result<Vec<u8>> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
    e.write_all(data)?;
    e.finish()
}