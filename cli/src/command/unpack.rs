use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error as IOError, ErrorKind, Result, Write};
use std::path::PathBuf;
use super::{cmd, data1, data2, data3, emphasize1, emphasize2, err, warn};
use super::mtklogo::{ColorMode, ContentType, FileInfo, LogoImage};
use super::mtklogo::utils::{image::ImageIO, z_lib};
use super::super::config::{Config, Format};

pub fn run_unpack(config: Config, slots: Option<Vec<usize>>, profile_name: &str,
                  mode: Option<&str>, flip: bool, zip: bool, check: bool,
                  path: PathBuf, output: PathBuf) -> Result<()> {
    // What is active profile?
    let maybe_profile =
        config.profiles.iter().find(|i| { i.name.eq(&profile_name) });
    let mut profile = match maybe_profile {
        Some(p) => Ok(p.clone()),
        None => Err(IOError::new(ErrorKind::InvalidData,
                                 format!("profile '{}' is not declared in configuration file", profile_name)))
    }?;
    // User may override color model.
    if let Some(model) = mode {
        profile = profile.with_color_model(String::from(model));
    };
    let mtk_color_model = ColorMode::by_name(&profile.color_model)?;
    println!("{} file {} with profile {}, color mode {}, flip orientation: {} to directory {}.",
             cmd("unpack"),
             emphasize1(path.display()),
             data1(profile_name),
             data2(format!("{}", mtk_color_model)),
             emphasize1(format!("{}", flip)),
             emphasize1(output.display()));

    // Opens the file
    let f = File::open(path)?;

    let format_provider = |sz: u32| profile.guess_format(sz, flip);

    // Reads through it.
    let mut reader = BufReader::new(f);
    // reads whole image in memory.
    let image = LogoImage::read(&mut reader)?;
    println!("logo image has {} slots", data1(image.blobs.len()));
    // let format_provider = &conf::guess_format;
    for (id, blob) in image.blobs.iter().enumerate() {
        let should_extract_zip = match slots {
            None => false,
            Some(ref s) => !s.contains(&id)
        };
        if check {
            check_logo(id, blob, zip || should_extract_zip, &mtk_color_model, &output, format_provider);
        } else {
            extract_logo(id, blob, zip || should_extract_zip, &mtk_color_model, &output, format_provider)?;
        }
    }
    Ok(())
}

fn extract_logo<F>(id: usize, blob: &Vec<u8>, zip: bool, color_mode: &ColorMode, outpath: &PathBuf, format_provider: F)
                   -> Result<()>
    where F: Fn(u32) -> Result<Format> {
    let info = FileInfo::from_info(id, zip, color_mode);
    // computes the output name.
    let output_file = outpath.join(info.filename());
    match &info.content_type {
        ContentType::Z =>
            export_raw(&info, &output_file, blob),
        ContentType::PNG(e) => {
            export_png(&info, &output_file, blob, color_mode, format_provider)
                .or_else(|er| {
                    println!("{} slot {} as {} because {}. Falling back to raw .z.",
                             warn("Could not export"),
                             data1(id), emphasize1(e),
                             err(er.description()), );
                    // invalidates names.
                    let info = FileInfo::from_info(id, true, color_mode);
                    // computes the output name.
                    let output_file = outpath.join(info.filename());
                    export_raw(&info, &output_file, blob)
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
            println!("slot {} is {} bytes and will be exported as raw zip to {}", id, blob.len(), &output_file.display()),
        ContentType::PNG(_) => {
            let exported = z_lib::inflate(blob as &[u8])
                // then resolves the couple (format, inflated).
                .and_then(|inflated| format_provider(inflated.len() as u32).map(|format| (format, inflated)))
                .and_then(|(format, inflated)| {
                    println!("slot {} is {} bytes ({} inflated) and will be exported as {}x{} image to {}",
                             id, blob.len(), inflated.len(), format.w, format.h, &output_file.display());
                    Ok(())
                });
            if let Some(er) = exported.err() {
                println!("{} slot {} ({} bytes) as an image : {}",
                         warn("Cannot export"), id, blob.len(), warn(er.description()));
            }
        }
    };
}

fn export_raw(info: &FileInfo, output_file: &PathBuf, blob: &Vec<u8>) -> Result<()> {
    println!("storing slot {} ({} bytes) to {} as raw zip .",
             data1(info.id),
             data2(blob.len()),
             emphasize1(output_file.display()));
    let mut f = File::create(output_file)?;
    f.write_all(blob)
}

fn export_png<F>(info: &FileInfo, output_file: &PathBuf, blob: &Vec<u8>, color_mode: &ColorMode, format_provider: F)
                 -> Result<()>
    where F: Fn(u32) -> Result<Format> {
    // inflates the zip
    z_lib::inflate(blob as &[u8])
        // then resolves the couple (format, inflated).
        .and_then(|inflated| format_provider(inflated.len() as u32).map(|format| (format, inflated)))
        .and_then(|(format, inflated)| {
            let file = File::create(output_file)?;
            let file_writer = BufWriter::new(file);
            println!("storing slot {} ({} bytes) to {} as {}x{} {} image.",
                     data1(info.id),
                     data2(blob.len()),
                     emphasize1(output_file.display()),
                     data3(format.w),
                     data3(format.h),
                     emphasize2(color_mode));
            color_mode.write_png(file_writer, &inflated, format.w, format.h)
        })
}
