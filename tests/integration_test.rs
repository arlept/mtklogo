extern crate byteorder;
#[macro_use]
extern crate lazy_static;
extern crate mtklogo;

// These integration tests do as little as checking:
// - whether the external crates (zlib/png) are fitted for the purpose of this program.
// - if I'm not too bad with image formats...

use byteorder::{ByteOrder, LittleEndian, BigEndian, ReadBytesExt};
use mtklogo::{LogoImage, same_bytes};
use mtklogo::utils::{image, load_raw};
use mtklogo::utils::z_lib;
use std::fs::File;
use std::io::{BufWriter, Result, Write, Read};
use std::path::PathBuf;

fn test_folder() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests");
    d
}

/// Compares rasters pixel for pixel, omitting a few bits specified by the mask.
fn compare_rasters<A: Read, B:Read>(mut a: A, mut b: B, pixels: usize, mask: u32){
    for _pixels in 0..pixels {
        let before = a.read_u32::<BigEndian>().unwrap();
        let after = b.read_u32::<BigEndian>().unwrap();
        assert_eq!(before & mask, after & mask);
    }
}
/// Utility function to get a grasp at the raw memory...
pub fn rgba_to_ppm<W: Write>(mut writer: W, data: &[u8], w: u32, h: u32) -> Result<()> {
    // see http://rosettacode.org/wiki/Bitmap/Write_a_PPM_file#Rust
    let header = format!("P6 {} {} 255\n", w, h);
    writer.write(header.as_bytes())?;
    let pixels = w * h;
    let mut offset = 0;
    for _ in 0..pixels {
        let rgb = [data[offset + 0], data[offset + 1], data[offset + 2]];
        writer.write_all(&rgb).unwrap();
        offset += 4;
    }
    Ok(())
}

fn png_to_raster_z(png: &[u8]) -> Vec<u8> {
    // Encodes it as RGBA.
    let (rgba, _, _) = image::png_to_rgba(png).unwrap();
    // zips it
    z_lib::deflate(&rgba).unwrap()
}

// just to avoid loading the samples too many times...
lazy_static! {
    static ref IMAGE1_PNG: Vec<u8> = {
        let file = File::open(test_folder().join("white-lotus-flower-bud.png")).unwrap();
        load_raw(&file).unwrap()
    };
    static ref IMAGE2_PNG: Vec<u8> = {
        let file = File::open(test_folder().join("boat-at-sunrise-1488476212bRg.png")).unwrap();
        load_raw(&file).unwrap()
    };
    static ref IMAGE1_Z: Vec<u8> = {
        png_to_raster_z(&IMAGE1_PNG)
    };
    static ref IMAGE2_Z: Vec<u8> = {
        png_to_raster_z(&IMAGE2_PNG)
    };
    static ref SAMPLE: LogoImage = {
        LogoImage::new_blobs(vec!(IMAGE1_Z.clone(), IMAGE2_Z.clone()))
    };
}

#[test]
fn dither_rgb565() {
    fn test_it<O:ByteOrder>(){
        // Gets an RGBA image
        let (rgba, w, h) = image::png_to_rgba(&IMAGE1_PNG as &[u8]).unwrap();
        // encodes it as rgb565
        let rgba565 = image::rgba_to_rgb565::<O,_>(&rgba as &[u8], w, h).unwrap();
        // It must be halved in size.
        assert_eq!(rgba.len() / 2, rgba565.len());
        // encodes it again as rgb
        let rgb_again = image::rgb565_to_rgba::<O>(&rgba565 as &[u8], w, h).unwrap();
        // Manual test:
        // saves it as a .ppm, you wil manually tell if it is a "good" dithering :)
        // let out = File::create("/tmp/mtklogo_rs_dither_rgb565.ppm").unwrap();
        // rgba_to_ppm(out, &rgb_again, w, h).unwrap();
        // ok, no, you won't do that each test... and don't want to have disk space filled
        // by my library, so we can do a few checks in memory:
        assert_eq!(rgb_again.len(), rgba.len());
        // let's check that we have the same images but with a little degradation...
        compare_rasters(&rgba as &[u8], &rgb_again as &[u8], (w * h) as usize, 0xF8FCF800);
    };
    // It should dither the same for big or little endian...
    test_it::<BigEndian>();
    test_it::<LittleEndian>();
}

/// We just test that we can read and write (serialize) a well crafted logo image file.
#[test]
fn can_explode() {
    const EXPECTED_IMAGES: usize = 2;
    assert_eq!(SAMPLE.blobs.len(), EXPECTED_IMAGES);
    assert_eq!(SAMPLE.table.logo_count, EXPECTED_IMAGES as u32);
    assert_eq!(SAMPLE.table.offsets.len(), EXPECTED_IMAGES);
    // I can re-assemble without crashing
    let mut writer = BufWriter::new(Vec::<u8>::with_capacity(2000000));
    SAMPLE.write(&mut writer).unwrap();
}


/// We check that compressing/decompressing the same data leads to 'equivalent' payloads.
#[test]
fn zlib_is_quite_symetric() {
    // Let's take some already compressed data.
    let blob1 = &IMAGE1_Z;
    // decompresses it
    let decompressed = z_lib::inflate(blob1 as &[u8]).unwrap();
    let recompressed = z_lib::deflate(&decompressed).unwrap();
    let grow_ratio = (100 * (recompressed.len() - blob1.len())) / blob1.len();
    println!("{} - {} - {}%", blob1.len(), recompressed.len(), grow_ratio);
    #[cfg(feature = "flate2")]
    let more_tests = || -> () {
        // With flate2, since it wraps the system zlib, it must be the exact same bytes!
        assert!(same_bytes(blob1, &recompressed));
    };
    #[cfg(feature = "libflate")]
    let more_tests = || -> () {
        // If we don't grow bigger than 15% the original size, it's OK...
        // In fact, it's 14%. That's a difference indeed.
        assert!(grow_ratio < 15);
    };
    more_tests();
}


/// We check that converting from raster to PNG back and forth does not change a single bit!
#[test]
fn png_is_not_lossy() {
    // takes our raw sample PNG.
    let (raster, w, h) = image::png_to_rgba(&IMAGE1_PNG as &[u8]).unwrap();
    assert_eq!(w, 720);
    assert_eq!(h, 1080);
    // manual test: if you want to check the decoded file.
    // let out = File::create("/tmp/png_exploded.ppm").unwrap();
    // rgba_to_ppm(out, &raster, w, h).unwrap();

    // converts it to PNG (assume it will occupy less than a quarter of memory...)
    let mut png_data = Vec::<u8>::with_capacity(raster.len() >> 2);
    // saves it again as a PNG
    println!("SAVE...");
    image::rgba_to_png(&mut png_data, &raster, w, h).unwrap();

    // converts it one more time to raster!
    let (raster_again, ww, hh) = image::png_to_rgba(&png_data as &[u8]).unwrap();
    assert_eq!(w, ww);
    assert_eq!(h, hh);

    // Hopefully: decode(encode(x)) = x...
    assert!(same_bytes(&raster, &raster_again));
}
