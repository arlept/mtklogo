pub use self::header::{MtkHeader, MtkType};
pub use self::logo::{LogoImage, LogoTable};

mod header;
mod logo;

trait StartExt {
    /// checks whether one byte array "starts with" another byte array,
    /// assuming the byte array is ascii characters and case is ignored.
    fn starts_with_ascii_ignore_case(&self, with: &Self) -> bool;
}

impl StartExt for [u8] {
    fn starts_with_ascii_ignore_case(&self, with: &[u8]) -> bool {
        if self.len() < with.len() { return false; }
        for i in 0..with.len() {
            let c = self[i];
            let w = with[i];
            if w.to_ascii_lowercase() != c && w.to_ascii_uppercase() != c{
                return false;
            }
        }
        return true;
    }
}
