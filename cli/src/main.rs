#[macro_use]
extern crate clap;
extern crate mtklogo;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use clap::{App, Arg, ArgMatches, SubCommand};
pub use config::{Config, ConfigColorMode, Format, Profile};
// re-exports entry points.
use std::path::PathBuf;
use std::process;
use std::env;

mod command;
mod config;


fn main() {
    // defines common args amongs commands.
    let slots_arg = Arg::with_name("slots")
        .help("Extracts image only these slots, other slot remain in raw .z format.")
        .value_name("slots")
        .takes_value(true)
        .long("slots")
        .conflicts_with("zip");

    let path_arg = Arg::with_name("path")
        .help("Path to input `logo.bin`")
        .required(true)
        .index(1);

    let prg = App::new("mtklogo")
        .version("0.0.1")
        .author("arlept, arnaud@lepoint.net")
        .about("Yet another Android Logo Customizer for MTK devices!\nIt packs or repacks images from an MTK `logo.bin` file.")
        .subcommand(SubCommand::with_name("unpack")
            .about("unpacks a logo image")
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
                .long("config"))
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
                .help("Sets images output directory")
                .value_name("output")
                .takes_value(true)
                .short("o")
                .long("output"))
            .arg(Arg::with_name("no-out")
                .help("Do not extract images, just checks image formats.")
                .short("n")
                .long("no-out")
                .conflicts_with("output"))
            .arg( &path_arg)
            .arg(&slots_arg)
        )

        .subcommand(SubCommand::with_name("explore")
            .about("unpacks a logo image with the specified format\n\
this is useful is you don't know the image format, you'll probably find out.")
            .arg(Arg::with_name("output")
                .help("Sets images output directory")
                .value_name("output")
                .takes_value(true)
                .short("o")
                .long("output"))
            .arg(Arg::with_name("width")
                .help("Image width in pixels")
                .value_name("width")
                .required(true)
                .takes_value(true)
                .short("w")
                .long("width"))
            .arg( &path_arg)
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
            .about("repacks a logo image")
            .arg(Arg::with_name("output")
                .value_name("output")
                .help("Path to output `logo.bin`")
                .required(true)
                .takes_value(true)
                .short("o")
                .long("output"))
            .arg(Arg::with_name("files")
                .help("Files to repack. Take care of packing them in the order they were unpacked!")
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

    if let Some(matches) = matches.subcommand_matches("unpack") {
        let config = solve_config(matches);
        let profile = matches.value_of("profile").unwrap_or("default");
        let mode = matches.value_of("mode");
        let flip = matches.is_present("flip");
        let zip = matches.is_present("zip");
        let check = matches.is_present("no-out");
        let path = solve_path(matches);
        let output = solve_output(matches);
        let slots = solve_slots(matches);
        command::run_unpack(config, slots, profile, mode, flip, zip, check, path, output);
    } else if let Some(matches) = matches.subcommand_matches("explore") {
        let path = solve_path(matches);
        let output = solve_output(matches);
        let width = value_t_or_exit!(matches, "width", u32);
        let slots = solve_slots(matches);
        command::run_explore(path, slots, output, width);
    } else if let Some(matches) = matches.subcommand_matches("repack") {
        let files = matches.values_of("files")
            .map(|vals| vals.collect::<Vec<_>>())
            .unwrap()
            .iter()
            .map(|f| PathBuf::from(f))
            .collect();
        let output = matches.value_of("output")
            .map(|o| PathBuf::from(o))
            .unwrap_or(PathBuf::default());
        let strip_alpha = matches.is_present("alpha");
        command::run_repack(output, files, strip_alpha);
    } else if let Some(matches) = matches.subcommand_matches("guess") {
        let size = value_t_or_exit!(matches, "size", usize);
        command::run_guess(size);
    }else {
        println!("{}", matches.usage());
        process::exit(1);
    }
}

fn solve_output(matches: &ArgMatches) -> PathBuf {
    let output = matches.value_of("output")
        .map(|o| PathBuf::from(o))
        .unwrap_or(env::current_dir().unwrap());
    if !output.as_path().exists() || !output.as_path().is_dir() {
        panic!(format!("output path: {:?} is not an existing directory.", output))
    }
    output
}

fn solve_config(matches: &ArgMatches) -> Config {
    match matches.value_of("config") {
        Some(c) => {
            Config::from_file(PathBuf::from(c).as_path()).unwrap()
        }
        None => Config::load().unwrap(),
    }
}

fn solve_path(matches: &ArgMatches) -> PathBuf {
    let path = matches.value_of("path")
        .map(|p| PathBuf::from(p))
        .unwrap();
    if !path.as_path().exists() || !path.as_path().is_file() {
        panic!(format!("logo file: {:?} does not exist", path))
    }
    path
}

fn solve_slots(matches: &ArgMatches) -> Option<Vec<usize>> {
    matches.value_of("slots").map(|s|{
        let tokens: Vec<&str> = s.split(',').collect();
        tokens.iter().map(|s| s.parse::<usize>().unwrap()).collect()
    })
}
