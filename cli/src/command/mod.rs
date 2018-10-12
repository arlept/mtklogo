extern crate ansi_term;
use self::ansi_term::{ANSIGenericString, Colour};
pub use self::explore::run_explore;
pub use self::guess::run_guess;
pub use self::repack::run_repack;
pub use self::unpack::run_unpack;
use std::fmt::Display;
use super::mtklogo;

mod unpack;
mod repack;
mod explore;
mod guess;

/// formats a command.
pub fn cmd<'a, I>(input: I) -> ANSIGenericString<'a, str>
    where I: Display + Sized{
    Colour::RGB(255,153,51).bold().paint(format!("{}", input))
}

/// formats a warning message.
pub fn warn<'a, I>(input: I) -> ANSIGenericString<'a, str>
    where I: Display + Sized{
    Colour::RGB(255,204,0).bold().paint(format!("{}", input))
}

/// formats an error message.
pub fn err<'a, I>(input: I) -> ANSIGenericString<'a, str>
    where I: Display + Sized{
    Colour::RGB(204,0,0).paint(format!("{}", input))
}

/// emphasizing on a text information.
pub fn emphasize1<'a, I>(input: I) -> ANSIGenericString<'a, str>
    where I: Display + Sized{
    Colour::RGB(204,204,0).paint(format!("{}", input))
}

/// emphasizing on a text information (variant)
pub fn emphasize2<'a, I>(input: I) -> ANSIGenericString<'a, str>
    where I: Display + Sized{
    Colour::RGB(102,153,0).paint(format!("{}", input))
}

/// emphasizing on a data.
pub fn data1<'a, I>(input: I) -> ANSIGenericString<'a, str>
    where I: Display + Sized{
    Colour::RGB(153,153,255).paint(format!("{}", input))
}

/// emphasizing on a data (variant).
pub fn data2<'a, I>(input: I) -> ANSIGenericString<'a, str>
    where I: Display + Sized{
    Colour::RGB(204,51,255).paint(format!("{}", input))
}

/// emphasizing on a data (variant).
pub fn data3<'a, I>(input: I) -> ANSIGenericString<'a, str>
    where I: Display + Sized{
    Colour::RGB(51,204,255).paint(format!("{}", input))
}




