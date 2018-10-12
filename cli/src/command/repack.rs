use std::fs::File;
use std::io::{BufWriter, Error as IOError, ErrorKind, Result};
use std::path::PathBuf;
use super::{cmd, data1, data2, emphasize1, emphasize2};
use super::mtklogo::{ContentType, FileInfo, LogoImage};
use super::mtklogo::utils::{image, image::ImageIO, load_raw, z_lib};

pub fn run_repack(outpath: PathBuf, files: Vec<PathBuf>, strip_alpha: bool) -> Result<()> {
    println!("{} {} files into {} stripping alpha: {}.",
             cmd("repack"),
             data1(files.len()),
             emphasize1(outpath.display()),
             data2(strip_alpha));

    // Reads input file meta information.
    let packable_files = reorder(files)?;
    // extracts blob data.
    let mut blobs = Vec::with_capacity(packable_files.len());
    for file in packable_files.iter() {
        blobs.push(import_logo(file, strip_alpha)?);
    }
    let count = blobs.len();
    let image = LogoImage::new_blobs(blobs);
    // saves it
    let mut writer = BufWriter::new(File::create(&outpath)?);
    image.write(&mut writer)?;
    println!("successfully repacked {} logos to {}", data1(count), emphasize1(outpath.display()));
    Ok(())
}

struct PackableFile {
    path: PathBuf,
    info: FileInfo,
}

fn import_logo(logo: &PackableFile, strip_alpha: bool) -> Result<Vec<u8>> {
    let file = File::open(&logo.path)?;
    match logo.info.content_type {
        ContentType::Z => {
            load_raw(file)
        }
        ContentType::PNG(ref color_mode) => {
            // loads png as rgba
            let (mut rgba, w, h) = image::png_to_rgba(file)?;
            // do we want to strip alpha?
            if strip_alpha { image::strip_alpha(&mut rgba) };
            // converts to device format.
            let device = color_mode.rgba_to_device(&rgba as &[u8], w, h)?;
            // zipped data.
            z_lib::deflate(&device)
        }
    }
}

fn reorder(files: Vec<PathBuf>) -> Result<Vec<PackableFile>> {
    // Analyses each file.
    let mut analyzed = Vec::with_capacity(files.len());
    for file in files.iter() {
        let path = file.as_path().to_str().ok_or_else(
            || IOError::new(ErrorKind::Other,
                            format!("file '{}' is not a possible path.", file.display())))?;
        let info = FileInfo::from_name(path)?;
        match &info.content_type {
            ContentType::Z =>
                println!("file {} is slot {} in raw z format.",
                         emphasize1(path), data1(info.id)),
            ContentType::PNG(p) =>
                println!("file {} is slot {} in {} format.",
                    emphasize1(path), data1(info.id), emphasize2(p)),
        }
        analyzed.push(PackableFile { path: file.clone(), info });
    }
    // returns the ordered list of files;
    analyzed.sort_by(|a, b| a.info.id.cmp(&b.info.id));
    Ok(analyzed)
}