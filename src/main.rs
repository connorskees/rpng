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
pub use crate::interlacing::Interlacing;
pub use png::PNG;

pub mod chunks;
mod common;
mod decoder;
mod encoder;
pub mod errors;
mod filter;
mod interlacing;
mod png;
mod utils;

#[allow(dead_code)]
fn main() -> Result<(), PNGDecodingError> {
    let png = PNG::open(std::env::args().nth(1).unwrap())?;
    dbg!(&png);
    let _pixels = png.pixels()?;
    // let mut f = File::create("fogkfkg.json")?;
    // f.write_all(serde_json::to_string(&_pixels.rows).unwrap().as_bytes())?;
    Ok(())
}
