extern crate clap;
extern crate mtklogo;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use clap::{App, Arg, ArgMatches, SubCommand};
use command::{emphasize1, err, warn};
pub use config::{Config, Format, Profile};
use std::env;
use std::error::Error;
use std::io::{Error as IOError, ErrorKind, Result as IOResult};
// re-exports entry points.
use std::path::PathBuf;

mod command;
mod config;

// logo generated from http://www.patorjk.com/software/taag/#p=display&h=1&v=3&f=Doom&t=mtklogo
const LOGO: &'static [u8] = include_bytes!("../resources/logo.txt");

fn main() {
    match wrapped_main() {
        Ok(()) => (),
        Err(e) => {
            println!("{}: {}", warn("error"), err(e.description()));
            std::process::exit(1);
        }
    }
}

fn wrapped_main() -> IOResult<()> {
    // defines common args amongst commands.
    let slots_arg = Arg::with_name("slots")
        .help("Extracts only these slots, other slot remain in raw .z format.")
        .value_name("slots")
        .takes_value(true)
        .long("slots")
        .conflicts_with("zip");

    let path_arg = Arg::with_name("path")
        .help("Path to input `logo.bin`")
        .required(true)
        .index(1)
        .validator(is_existing_file);

    let prg = App::new("mtklogo")
        .version("0.1.2")
        .author("arlept, arnaud@lepoint.net")
        .about("Yet another Android Logo Customizer for MTK devices!\nIt packs or repacks images from an MTK `logo.bin` file.")
        .subcommand(SubCommand::with_name("unpack")
            .about("Unpacks a logo image")
            .arg(Arg::with_name("profile")
                .help("Uses an alternative profile name")
                .value_name("profile")
                .short("p")
                .long("profile"))
            .arg(Arg::with_name("config")
                .help("Uses an alternative configuration file")
                .value_name("configfile")
                .takes_value(true)
                .short("c")
                .long("config")
                .validator(is_existing_file))
            .arg(Arg::with_name("mode")
                .help("Overrides profile's color mode")
                .value_name("mode")
                .short("m")
                .long("mode"))
            .arg(Arg::with_name("flip")
                .help("Flips orientation")
                .short("f")
                .long("flip"))
            .arg(Arg::with_name("zip")
                .help("Do not convert to png, extract as plain .z file")
                .short("z")
                .long("zip")
                .conflicts_with("slots"))
            .arg(Arg::with_name("output")
                .help("Sets images output path")
                .value_name("output")
                .takes_value(true)
                .short("o")
                .long("output")
                .validator(is_existing_directory))
            .arg(Arg::with_name("no-out")
                .help("Do not extract images, just checks image formats.")
                .short("n")
                .long("no-out")
                .conflicts_with("output"))
            .arg(&path_arg)
            .arg(&slots_arg)
        )

        .subcommand(SubCommand::with_name("explore")
            .about("Unpacks a logo image with the specified format\n\
this is useful is you don't know the image format, you'll probably find out.")
            .arg(Arg::with_name("output")
                .help("Sets images output directory")
                .value_name("output")
                .takes_value(true)
                .short("o")
                .long("output")
                .validator(is_existing_directory))
            .arg(Arg::with_name("width")
                .help("Image width in pixels")
                .value_name("width")
                .required(true)
                .takes_value(true)
                .short("w")
                .long("width"))
            .arg(&path_arg)
            .arg(&slots_arg)
        )

        .subcommand(SubCommand::with_name("guess")
            .about("Tries to guess an image dimension knowing its buffer size.\n\
Note: the program may be very slow if your input size is a large prime number!")
            .arg(Arg::with_name("size")
                .help("Image size in bytes")
                .value_name("size")
                .required(true)
                .takes_value(true)
                .short("s")
                .long("size"))
        )

        .subcommand(SubCommand::with_name("repack")
            .about("Repacks a logo image")
            .arg(Arg::with_name("output")
                .value_name("output")
                .help("Path to output `logo.bin`")
                .required(true)
                .takes_value(true)
                .short("o")
                .long("output"))
            .arg(Arg::with_name("files")
                .help("Files to repack. Take care of specifying the exact set of files!")
                .value_name("files")
                .multiple(true)
                .required(true))
            .arg(Arg::with_name("alpha")
                .help("Strips Alpha channel, assume image is opaque")
                .short("a")
                .long("alpha"))
        )
    ;
    let matches = prg.get_matches();

    println!("{}", emphasize1(String::from_utf8_lossy(LOGO)));

    if let Some(matches) = matches.subcommand_matches("unpack") {
        let config = solve_config(matches)?;
        let profile = matches.value_of("profile").unwrap_or("default");
        let mode = matches.value_of("mode");
        let flip = matches.is_present("flip");
        let zip = matches.is_present("zip");
        let check = matches.is_present("no-out");
        let path = solve_path(matches)?;
        let output = solve_output(matches)?;
        let slots = solve_slots(matches)?;

        command::run_unpack(config, slots, profile, mode, flip, zip, check, path, output)
    } else if let Some(matches) = matches.subcommand_matches("explore") {
        let path = solve_path(matches)?;
        let output = solve_output(matches)?;
        let width = parse_or_error::<u32>(matches, "width")?;
        let slots = solve_slots(matches)?;
        command::run_explore(path, slots, output, width)
    } else if let Some(matches) = matches.subcommand_matches("repack") {
        let maybe_files = matches.values_of("files")
            .map(|vals| vals.collect::<Vec<_>>());
        let files = maybe_files.map_or_else(
            || Err(IOError::new(ErrorKind::Other, "no files to convert")),
            |f| Ok(f))?;
        let paths = files
            .iter()
            .map(|f| PathBuf::from(f))
            .collect();
        let output = matches.value_of("output")
            .map(|o| PathBuf::from(o))
            .unwrap_or(PathBuf::default());
        let strip_alpha = matches.is_present("alpha");
        command::run_repack(output, paths, strip_alpha)
    } else if let Some(matches) = matches.subcommand_matches("guess") {
        let size = parse_or_error::<usize>(matches, "size")?;
        command::run_guess(size)
    } else {
        println!("{}", matches.usage());
        Err(IOError::new(ErrorKind::InvalidInput, "unrecognized command arguments."))
    }
}

fn value_or_error(matches: &ArgMatches, label: &str) -> IOResult<String> {
    matches.value_of(label).map_or_else(
        || Err(IOError::new(ErrorKind::InvalidInput, format!("'{}' unspecified.", label))),
        |v| Ok(String::from(v)))
}

fn parse_or_error<T>(matches: &ArgMatches, label: &str) -> IOResult<T>
    where T: std::str::FromStr {
    value_or_error(matches, label)
        .and_then(|v| v.parse::<T>().map_err(|_| IOError::new(
            ErrorKind::InvalidInput, format!("'{}' has not expected format", label))))
}


fn solve_output(matches: &ArgMatches) -> IOResult<PathBuf> {
    value_or_error(matches, "output")
        .map(|o| PathBuf::from(o))
        .or_else(|_| env::current_dir())
}

fn solve_config(matches: &ArgMatches) -> IOResult<Config> {
    match matches.value_of("config") {
        Some(c) => {
            Config::from_file(PathBuf::from(c).as_path())
        }
        None => Config::load()
    }
}

fn solve_path(matches: &ArgMatches) -> IOResult<PathBuf> {
    value_or_error(matches, "path").map(|p| PathBuf::from(p))
}

fn solve_slots(matches: &ArgMatches) -> IOResult<Option<Vec<usize>>> {
    match matches.value_of("slots") {
        Some(slots) => {
            let tokens: Vec<&str> = slots.split(',').collect();
            let mut sizes: Vec<usize> = Vec::with_capacity(tokens.len());
            for s in tokens.iter() {
                let value = s.parse::<usize>()
                    .map_err(|_| IOError::new(
                        ErrorKind::InvalidInput, format!("'{}' is not an integer", s)))?;
                sizes.push(value);
            }
            Ok(Some(sizes))
        }
        None => Ok(None)
    }
}

fn is_existing_directory(val: String) -> Result<(), String> {
    let path = PathBuf::from(val);
    if path.exists() && path.is_dir() {
        Ok(())
    } else {
        Err(String::from("must be an existing directory."))
    }
}

fn is_existing_file(val: String) -> Result<(), String> {
    if PathBuf::from(val).exists() {
        Ok(())
    } else {
        Err(String::from("must be an existing file."))
    }
}
