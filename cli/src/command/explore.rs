use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Result};
use std::path::PathBuf;
use super::{cmd, data1, data2, data3, emphasize1, emphasize2, err, warn};
use super::mtklogo::{ColorMode, ContentType, FileInfo, LogoImage};
use super::mtklogo::utils::{image::ImageIO, z_lib};

pub fn run_explore(path: PathBuf, slots: Option<Vec<usize>>, output: PathBuf, width: u32) -> Result<()> {
    println!("{} file {}, width hint {}, saving to {}",
             cmd("explore"),
             emphasize1(path.display()),
             data1(width),
             emphasize1(output.display()));
    // Opens the file
    let f = File::open(path)?;
    // Reads through it.
    let mut reader = BufReader::new(f);
    // reads whole image in memory.
    let image = LogoImage::read(&mut reader)?;
    // let format_provider = &conf::guess_format;
    for (id, blob) in image.blobs.iter().enumerate() {
        let should_extract = match slots {
            None => true,
            Some(ref s) => s.contains(&id)
        };
        if should_extract {
            match extract_logo(id, blob, width, &output) {
                Err(e) => println!(
                    "{} {} : {}",
                    warn("Could not explore slot"),
                    data1(id),
                    err(e.description())),
                _ => (),
            }
        }
    }
    Ok(())
}

fn extract_logo(id: usize, blob: &Vec<u8>, width: u32, outpath: &PathBuf)
                -> Result<()> {
    // inflates the blob
    let inflated = z_lib::inflate(blob as &[u8])?;
    // how many bytes is it?
    let pixels = inflated.len() as u32;
    let extract = |mode: &ColorMode| -> Result<()>{
        // given a width, there is a maximum height depending on the image resolution and weight.
        let height = pixels / (width * mode.bytes_per_pixel());
        if height == 0 {
            println!("slot {} has {} data bytes, height would be 0, it cannot be {} wide in {}",
                     data1(id), data1(inflated.len()), data1(width), emphasize1(&mode));
            return Ok(()); // sort of...
        }
        let total_size = height * width * mode.bytes_per_pixel();
        if total_size != pixels {
            // PNG encoder would complain that ''destination and source slices have different lengths''
            println!("slot {} has {} data bytes, {}w * {}h * {}bpp (={}) would not match",
                     data1(id), data2(inflated.len()), data3(width), data3(height),
                     data1(mode.bytes_per_pixel()), data2(total_size));
            return Ok(()); // sort of...
        }

        let info = FileInfo { id, content_type: ContentType::PNG(mode.clone()) };
        let filename = format!("explore_{}", info.filename());
        println!("slot {} is {} bytes. It could be {}x{} {}, view it as {}",
                 data1(id), data2(pixels), data3(width), data3(height),
                 emphasize1(mode), emphasize2(&filename));
        let writer = File::create(outpath.join(&filename))?;
        let status = mode.write_png(writer, &inflated, width, height);
        if let Err(e) = status {
            println!("{} {} as {}x{} {}: {}",
                     warn("Could not extract slot"),
                     data1(id), data3(width), data3(height),
                     emphasize1(mode), err(e));
        }
        // we don't fail.
        Ok(())
    };
    // Attempt to export it in many formats.
    for mode in ColorMode::enumerate().iter() {
        extract(mode)?;
    }
    // We don't fail.
    Ok(())
}
