pub use self::header::{MtkHeader, MtkType};
pub use self::logo::{LogoImage, LogoTable};

mod header;
mod logo;

/// checks whether two bytes arrays are exactly the same.
pub fn same_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }
    return true;
}

/// checks whether one byte array "starts with" another byte array.
pub fn starts_with_bytes(a: &[u8], b: &[u8]) -> bool {
    if b.len() < a.len() { return false; }
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }
    return true;
}