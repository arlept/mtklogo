use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{BufWriter, Error as IOError, ErrorKind, Read, Result, Write};
use super::starts_with_bytes;

#[derive(Copy, Clone, Debug)]
/// An MTK image header.
pub struct MtkHeader {
    pub size: u32,
    pub mtk_type: MtkType,
}

#[derive(Copy, Clone, Debug)]
pub enum MtkType {
    RECOVERY,
    ROOTFS,
    KERNEL,
    LOGO,
}

impl MtkType {
    /// Tests whether the specified "magic bytes" correspond to some possible mtk image type.
    fn from_bytes(bytes: &[u8]) -> Option<MtkType>{
        if starts_with_bytes("RECOVERY".as_bytes(), bytes) {return Some(MtkType::RECOVERY)}
        if starts_with_bytes("ROOTFS".as_bytes(), bytes) {return Some(MtkType::ROOTFS)}
        if starts_with_bytes("KERNEL".as_bytes(), bytes) {return Some(MtkType::KERNEL)}
        if starts_with_bytes("LOGO".as_bytes(), bytes) {return Some(MtkType::LOGO)}
        None
    }
}

impl MtkHeader {
    pub const SIZE: usize = 512;
    pub const FILL: u8 = 0xFF;
    pub const MAGIC: u32 = 0x88168858;

    /// Reads an header.
    pub fn read<R: Read>(reader: &mut R) -> Result<MtkHeader> {
        let magic: u32 = reader.read_u32::<BigEndian>()?;
        // Assert is magic flag.
        if magic != Self::MAGIC {
            return Err(IOError::new(ErrorKind::InvalidData, "missing magic number"));
        }
        let size: u32 = reader.read_u32::<LittleEndian>()?;
        let mut typ = [0 as u8; 32];
        reader.read_exact(&mut typ)?;
        let mtk_type = MtkType::from_bytes(&typ).ok_or(
            IOError::new(ErrorKind::InvalidData, "Missing MTK Header type.")
        )?;

        let mut remainder = [0 as u8; 472];
        reader.read_exact(&mut remainder)?;
        // Checking the remainder is filled
        for i in 0..(remainder.len()) {
            if remainder[i] != 0xFF {
                return Err(IOError::new(
                    ErrorKind::InvalidData,
                    "unused zone not filled, probably not an mtk header"));
            }
        }
        Ok(MtkHeader { size, mtk_type })
    }

    /// Writes this header to the specified writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<BigEndian>(Self::MAGIC)?;
        writer.write_u32::<LittleEndian>(self.size)?;
        let mut imagetype = [0 as u8; 32];
        {
            let mut type_writer = BufWriter::new(&mut imagetype as &mut [u8]);
            let label = match self.mtk_type {
                MtkType::LOGO => "LOGO",
                MtkType::RECOVERY => "RECOVERY",
                MtkType::KERNEL => "KERNEL",
                MtkType::ROOTFS => "ROOTFS"
            };
            type_writer.write_all(label.as_bytes())?;
        }

        writer.write_all(&imagetype)?;
        let remainder = [Self::FILL as u8; 472];
        writer.write_all(&remainder)?;
        Ok(())
    }
}

