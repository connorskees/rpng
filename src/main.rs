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

#![forbid(unsafe_code, missing_debug_implementations)]

#[cfg(feature = "serialize")]
use serde_json;
#[cfg(feature = "serialize")]
use std::fs::File;
#[cfg(feature = "serialize")]
use std::io::Write;

pub use crate::common::*;
pub use crate::decoder::PngDecoder;
use crate::errors::*;
pub use crate::filter::*;
pub use crate::interlacing::Interlacing;
pub use png::Png;

pub mod chunks;
mod common;
mod decoder;
mod encoder;
pub mod errors;
mod filter;
mod interlacing;
mod png;

#[allow(dead_code)]
fn main() -> Result<(), PngDecodingError> {
    let png = Png::open(std::env::args().nth(1).unwrap())?;
    dbg!(&png);
    let pixels = png.pixels()?;
    // dbg!(&pixels);

    #[cfg(feature = "serialize")]
    {
        let mut f = File::create("fogkfkg.json")?;
        f.write_all(serde_json::to_string(&pixels.rows).unwrap().as_bytes())?;
    }

    Ok(())
}
