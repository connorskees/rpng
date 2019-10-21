//! # Library for working with PNG files
//! 
//! ## Currently supports
//! |                  | Decoding                                             | Encoding |
//! |------------------|------------------------------------------------------|----------|
//! | Bit Depth        | 1, 8                                                 |          |
//! | Color Type       | RGB, RGBA, Indexed partial support for Lum and LumA  |          |
//! | Filtering        | All filter methods                                   |          |
//! | Interlacing      | None                                                 |          |
//! | Ancillary Chunks | pHYs, tEXt, iTXt, bKGD, gAMA, sRGB, cHRM, iCCP, sBIT |          |

#![deny(unsafe_code, missing_debug_implementations)]

#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::Write;

#[cfg(test)]
use serde_json;


pub use crate::common::*;
pub use crate::decoder::PNGDecoder;
use crate::errors::*;
pub use crate::filter::*;
pub use crate::interlacing::{Interlacing};
pub use png::PNG;

mod common;
mod decoder;
mod encoder;
pub mod errors;
pub mod chunks;
mod filter;
mod interlacing;
mod png;
mod utils;

#[allow(dead_code)]
fn main() -> Result<(), PNGDecodingError> {
    // let png = PNG::open(r"C:\Users\Connor\Downloads\lindajsummers.png")?;
    // let png = PNG::open(r"tests\test_images\7x7.png")?;
    // use crate::encoder::save;
    // println!("{:?}", png);
    // save(png, "hi.png")?;
    let png2 = PNG::open(r"C:\Users\Connor\Downloads\Screenshot_20190823-000859.png")?;
    // let png2 = PNG::open(r"C:\Users\Connor\Downloads\unnamed.png")?;
    println!("{:?}", png2);

    // let png = PNG::open(r"C:\Users\Connor\Documents\Atom Projects\rust\rpng\tests\test_images\pngsuite\cdhn2c08.png")?;
    // let png = PNG::open(r"tests\test_images\7x7-adam7.png")?;
    // png.iccp_profile()?;
    // let _pixels = png.pixels()?;
    // println!("{:?}", _pixels);
    Ok(())
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use super::*;

    #[test]
    fn test() -> Result<(), PNGDecodingError> {
        let png = PNG::open(r"C:\Users\Connor\Documents\Atom Projects\rust\rpng\tests\test_images\pngsuite\bgbn4a08.png")?;
        println!("{:?}", png);
        let _pixels = png.pixels()?;
        let mut f = File::create("fogkfkg.json")?;
        f.write_all(serde_json::to_string(&_pixels.rows).unwrap().as_bytes())?;
        Ok(())
    }
}