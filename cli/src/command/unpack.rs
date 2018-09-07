use ConfigColorMode;
use serde_yaml;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result, Write};
use std::path::PathBuf;
use super::mtklogo::{ColorMode, ContentType, FileInfo, LogoImage};
use super::mtklogo::utils::{image::ImageIO, z_lib};
use super::super::config::{Config, Format};

// TODO: verbose, dump logo table, etc.
pub fn run_unpack(config: Config, slots: Option<Vec<usize>>, profile_name: &str,
                  mode: Option<&str>, flip: bool, zip: bool, check: bool, path: PathBuf, output: PathBuf) {
    // What is active profile?
    let maybe_profile = config.profiles.iter().find(|i| { i.name.eq(&profile_name) });
    if maybe_profile.is_none() {
        panic!(format!("profile '{:?}' is not declared in configuration file", profile_name));
    }
    let mut profile = maybe_profile.unwrap().clone();
    // User may override color model.
    if let Some(ref str) = mode {
        let color_model: ConfigColorMode = serde_yaml::from_str(str).unwrap();
        profile = profile.with_color_model(color_model);
    };
    let color_model = &profile.color_model;
    println!("unpacking with profile {:?}, color mode {:?}, flip orientation: {:?}.", profile_name, color_model, flip);

    // Opens the file
    let f = File::open(path).unwrap();
    // Reads through it.
    let mut reader = BufReader::new(f);

    let format_provider = |sz: u32| {
        let format = profile.guess_format(sz, flip);
        if let Ok(ref f) = format {
            println!("{:?} bytes yields {:?}*{:?} for profile {:?} and mode {:?}", sz, f.w, f.h, profile_name, color_model);
        }
        format
    };

    // reads whole image in memory.
    let image = LogoImage::read(&mut reader).unwrap();
    // let format_provider = &conf::guess_format;
    for (id, blob) in image.blobs.iter().enumerate() {
        let should_extract_zip = match slots {
            None => false,
            Some(ref s) => !s.contains(&id)
        };
        if check {
            check_logo(id, blob, zip || should_extract_zip, &color_model.to_mtk(), &output, format_provider);
        } else {
            extract_logo(id, blob, zip || should_extract_zip, &color_model.to_mtk(), &output, format_provider).unwrap();
        }
    }
}

fn extract_logo<F>(id: usize, blob: &Vec<u8>, zip: bool, color_mode: &ColorMode, outpath: &PathBuf, format_provider: F)
                   -> Result<()>
    where F: Fn(u32) -> Result<Format> {
    let info = FileInfo::from_info(id, zip, color_mode);
    // computes the output name.
    let output_file = outpath.join(info.filename());
    match info.content_type {
        ContentType::Z =>
            export_raw(&output_file, blob),
        ContentType::PNG(e) => {
            export_png(&output_file, blob, color_mode, format_provider)
                .or_else(|er| {
                    println!("Error : {:?}.\nCould not export as {:?}, falling back to raw .z", er, e);
                    // invalidates names.
                    let info = FileInfo::from_info(id, true, color_mode);
                    // computes the output name.
                    let output_file = outpath.join(info.filename());
                    export_raw(&output_file, blob)
                })
        }
    }
}

fn check_logo<F>(id: usize, blob: &Vec<u8>, zip: bool, color_mode: &ColorMode, outpath: &PathBuf, format_provider: F)
    where F: Fn(u32) -> Result<Format> {
    let info = FileInfo::from_info(id, zip, color_mode);
    // computes the output name.
    let output_file = outpath.join(info.filename());
    match info.content_type {
        ContentType::Z =>
            println!("slot {} is {} bytes and will be exported as raw zip to {:?}", id, blob.len(), &output_file),
        ContentType::PNG(_) => {
            let exported = z_lib::inflate(blob as &[u8])
                // then resolves the couple (format, inflated).
                .and_then(|inflated| format_provider(inflated.len() as u32).map(|format| (format, inflated)))
                .and_then(|(format, inflated)| {
                    println!("slot {} is {} bytes ({} inflated) and will be exported as {}x{} image to {:?}",
                             id, blob.len(), inflated.len(), format.w, format.h, &output_file);
                    Ok(())
                });
            if let Some(er) = exported.err() {
                println!("slot {} is {} bytes and cannot be exported as an image : {:?}", id, blob.len(), er);
            }
        }
    };
}

fn export_raw(output_file: &PathBuf, blob: &Vec<u8>) -> Result<()> {
    let mut f = File::create(output_file)?;
    f.write_all(blob)
}

fn export_png<F>(output_file: &PathBuf, blob: &Vec<u8>, color_mode: &ColorMode, format_provider: F)
                 -> Result<()>
    where F: Fn(u32) -> Result<Format> {
    // inflates the zip
    z_lib::inflate(blob as &[u8])
        // then resolves the couple (format, inflated).
        .and_then(|inflated| format_provider(inflated.len() as u32).map(|format| (format, inflated)))
        .and_then(|(format, inflated)| {
            let file = File::create(output_file)?;
            let file_writer = BufWriter::new(file);
            color_mode.write_png(file_writer, &inflated, format.w, format.h)
        })
}
