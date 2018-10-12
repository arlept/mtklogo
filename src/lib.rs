extern crate byteorder;

pub use mtk::{LogoImage, LogoTable, MtkHeader, MtkType, same_bytes};
use std::fmt;
use std::fmt::Display;
use std::io::{Error as IOError, ErrorKind, Result};

// MTK Structures
pub mod mtk;
// I/O Utilities (zlib, png)
pub mod utils;

#[derive(Debug, Clone, PartialEq)]
/// Device may encode data in little or big endian.
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, Clone, PartialEq)]
/// Supported color modes (how pixels are encoded).
pub enum ColorMode {
    Rgba(Endian),
    Bgra(Endian),
    Rgb565(Endian),
}

static SUPPORTED_MODES: [(ColorMode, &str); 6] = [
    (ColorMode::Rgba(Endian::Big), "rgbabe"),
    (ColorMode::Rgba(Endian::Little), "rgbale"),
    (ColorMode::Bgra(Endian::Big), "bgrabe"),
    (ColorMode::Bgra(Endian::Little), "bgrale"),
    (ColorMode::Rgb565(Endian::Big), "rgb565be"),
    (ColorMode::Rgb565(Endian::Little), "rgb565le")];

impl ColorMode {
    /// Lists all managed color modes.
    pub fn enumerate<'a>() -> Vec<&'a ColorMode> {
        // concision over performance...
        SUPPORTED_MODES.iter().map(|(mode, _name)| mode).collect()
    }
    pub fn by_name<'a>(name: &str) -> Result<&'a ColorMode> {
        // concision over performance...
        SUPPORTED_MODES.iter()
            .find(|(_, n)| name.eq(*n))
            .map(|(mode, _)| mode)
            .ok_or_else(|| IOError::new(
                ErrorKind::InvalidInput, format!("{} is not a color mode", name)))
    }
}

impl Display for ColorMode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // concision over performance...
        match SUPPORTED_MODES.iter().find(|(mode, _)| mode.eq(self)) {
            Some((_, name)) => {
                fmt.write_str(name)
            }
            None => fmt.write_str("ColorMode?") // should be unreachable.
        }
    }
}

impl ColorMode {
    /// How many bytes are required to encode a single pixel?
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            &ColorMode::Rgba(_) => 4,
            &ColorMode::Bgra(_) => 4,
            &ColorMode::Rgb565(_) => 2,
        }
    }
}

#[derive(Debug, Clone)]
/// What kind of content does a file hold?
pub enum ContentType {
    /// Plain zlib encoded data,
    Z,
    /// A PNG image which is meant for the specified color mode of the device.
    PNG(ColorMode),
}

impl ContentType {
    /// Given a file name, can we say which Content Type it is?
    pub fn from_name(name: &str) -> Option<Self> {
        match SUPPORTED_MODES.iter().find(|(_, mode_name)| {
            name.ends_with(&format!("{}.png", mode_name))
        }).map(|(mode, _)| mode.clone()) {
            Some(mode) => Some(ContentType::PNG(mode)),
            None => if name.ends_with("raw.z") { Some(ContentType::Z) } else { None }
        }
    }
}

#[derive(Debug, Clone)]
/// A structure to gather information about an image/slot.
pub struct FileInfo {
    pub id: usize,
    pub content_type: ContentType,
}

impl FileInfo {
    /// How would we name the file for this image/slot?
    pub fn filename(&self) -> String {
        match self.content_type {
            ContentType::Z => format!("logo_{:03}_raw.z", self.id),
            ContentType::PNG(ref mode) => format!("logo_{:03}_{}.png", self.id, mode),
        }
    }

    pub fn from_info(id: usize, zip: bool, color_model: &ColorMode) -> Self {
        FileInfo {
            id,
            content_type: if zip { ContentType::Z } else { ContentType::PNG(color_model.clone()) },
        }
    }

    pub fn from_name(name: &str) -> Result<FileInfo> {
        let tokens: Vec<&str> = name.split('_').collect();
        // Extracting id in "xxx_id_yyy", as the 'middle' token in ['xxx', id, 'yyy']
        let id = tokens.get(1).map_or_else(
            // no middle token
            || Err(IOError::new(ErrorKind::InvalidInput, "cannot find '_id_' token")),
            |middle_id| Ok(middle_id)).and_then(|middle_id| middle_id.parse::<usize>().map_err(
            |_| IOError::new(ErrorKind::InvalidInput, "cannot parse '_id' token")))?;
        if let Some(content_type) = ContentType::from_name(name) {
            Ok(FileInfo { id, content_type })
        } else {
            Err(IOError::new(ErrorKind::InvalidInput,
                             format!(
                                 "file '{}' does not look like a .z or a supported png format", name)))
        }
    }
}
