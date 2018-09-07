use std::fs::File;
use std::io::{BufWriter, Result};
use std::path::PathBuf;
use super::mtklogo::{ContentType, FileInfo, LogoImage};
use super::mtklogo::utils::{image::ImageIO, image, load_raw, z_lib};

pub fn run_repack(outpath: PathBuf, files: Vec<PathBuf>, strip_alpha: bool) {
    println!("outpath {:?} files {:?}", outpath, files);
    // Reads input file meta information.
    let packable_files = reorder(files);
    // extracts blob data.
    let mut blobs = Vec::with_capacity(packable_files.len());
    for file in packable_files.iter() {
        // TODO: dirty unwrap.
        blobs.push(import_logo(file, strip_alpha).unwrap());
    }
    let count = blobs.len();
    let image = LogoImage::new_blobs(blobs);
    // saves it
    let mut writer = BufWriter::new(File::create(&outpath).unwrap());
    image.write(&mut writer).unwrap();
    println!("successfully repacked {:?} logos to {:?}", count, outpath);
}

#[derive(Debug)]
struct PackableFile {
    path: PathBuf,
    info: FileInfo,
}

fn import_logo(logo: &PackableFile, strip_alpha: bool) -> Result<Vec<u8>> {
    println!("importing {:?}", logo);
    let file = File::open(&logo.path)?;
    match logo.info.content_type {
        ContentType::Z => {
            load_raw(file)
        }
        ContentType::PNG(ref color_mode) => {
            // loads png as rgba
            let (mut rgba, w, h) = image::png_to_rgba(file)?;
            // do we want to strip alpha?
            if strip_alpha {image::strip_alpha(&mut rgba)};
            // converts to device format.
            let device = color_mode.rgba_to_device(&rgba as &[u8], w, h)?;
            // zipped data.
            z_lib::deflate(&device)

        }
    }
}

fn reorder(files: Vec<PathBuf>) -> Vec<PackableFile> {
    // Analyses each file.
    let mut analyzed: Vec<PackableFile> = files.iter().map(|file| {
        // TODO: kill unwrap, result.
        let info = FileInfo::from_name(file.as_path().to_str().unwrap()).unwrap();
        println!("{:?} is a  {:?}", file, info);
        PackableFile { path: file.clone(), info }
    }).collect();
    // returns the ordered list of files;
    analyzed.sort_by(|a, b| a.info.id.cmp(&b.info.id));
    analyzed
}