extern crate png;

use std::io::{Read, Result};
use super::byteorder;

pub mod image;

#[cfg(feature = "libflate")]
pub mod z_lib_libflate;

#[cfg(feature = "libflate")]
pub use self::z_lib_libflate as z_lib;

#[cfg(feature = "flate2")]
pub mod z_lib_flate2;

#[cfg(feature = "flate2")]
pub use self::z_lib_flate2 as z_lib;

pub fn load_raw<R: Read>(mut reader: R) -> Result<(Vec<u8>)> {
    let mut buf = Vec::with_capacity(8192); // no idea how to size it.
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

