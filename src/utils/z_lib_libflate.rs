extern crate libflate;

use self::libflate::zlib::{Decoder, Encoder};
use std::io::{Read, Result, Write};

// It's just a thin wrapper around 'libflate'.
// If you want a "pure rust" program (and don't require high compression) this is the library to use.

pub fn inflate<R>(data: R) -> Result<Vec<u8>> where
    R: Read{
    let mut decoder = Decoder::new(data)?;
    let mut uncompressed = Vec::new();
    decoder.read_to_end(&mut uncompressed).map(|_sz| uncompressed)
}

pub fn deflate(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = Encoder::new(Vec::new())?;
    encoder.write_all(&data[..])?;
    encoder.finish().into_result()
}
