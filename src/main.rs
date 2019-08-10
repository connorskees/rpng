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
use crate::errors::*;


pub use crate::common::*;
pub use crate::chunks::{IHDR, PLTE, pHYs, Unit, UnrecognizedChunk, AncillaryChunks, ICCProfile};
pub use crate::decoder::PNGDecoder;
pub use crate::filter::{FilterMethod, FilterType};
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

#[allow(dead_code)]
fn main() -> Result<(), PNGDecodingError> {
    println!("{:?}", png);
    Ok(())
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use super::*;

    #[test]
    fn test() -> Result<(), PNGDecodingError> {
        let png = PNG::from_path(r"C:\Users\Connor\Downloads\SF.LogoChop-transparent.png")?;
        println!("{:?}", png);
        let _pixels = png.pixels()?;
        let mut f = File::create("fogkfkg.json")?;
        f.write_all(serde_json::to_string(&_pixels.rows).unwrap().as_bytes())?;
        Ok(())
    }
}


}