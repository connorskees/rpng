//! Module for working with PNG files

#![allow(dead_code)]
#![deny(unsafe_code, missing_debug_implementations)]

use std::io::{BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::convert::AsRef;
use std::{fmt, fs, str};
use std::vec::Vec;

use flate2::bufread::ZlibDecoder;
// use serde_json;

use chunks::{IHDR, PLTE, UnrecognizedChunk, AncillaryChunks};
pub use common::{Bitmap, BitDepth, ColorType, CompressionType, Unit};
pub use filter::{FilterMethod, FilterType};
pub use interlacing::{Interlacing};
pub use errors::PNGDecodingError;
pub use decoder::PNGDecoder;

mod common;
mod decoder;
mod errors;
pub mod chunks;
mod filter;
mod interlacing;

const FILE_NAME: &str = "redrect";

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PNG {
    ihdr: IHDR,
    plte: Option<PLTE>,
    idat: Vec<u8>,
    unrecognized_chunks: Vec<UnrecognizedChunk>,
    ancillary_chunks: AncillaryChunks,
}

impl fmt::Debug for PNG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "PNG {{\n    ihdr: {:?}\n    plte: {:?}\n    data: {} bytes (compressed)\n    unrecognized_chunks: {:#?}\n    ancillary_chunks: {:#?}}}",
            self.ihdr, self.plte.as_ref(), self.idat.len(), self.unrecognized_chunks, self.ancillary_chunks
        )
    }
}

impl PNG {
    pub fn from_path<S: AsRef<Path>>(file_path: S) -> Result<Self, PNGDecodingError> {
        let file_size: usize = fs::metadata(&file_path)?.len() as usize;
        PNGDecoder::read(BufReader::with_capacity(file_size, File::open(file_path)?))
    }

    pub fn pixels(&self) -> Result<Bitmap<u16>, PNGDecodingError> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut zlib = ZlibDecoder::new(&self.idat as &[u8]);
        let buf_len = zlib.read_to_end(&mut buffer)?;
        if buf_len == 0 {
            return Err(PNGDecodingError::ZeroLengthIDAT("no pixel data provided"));
        }


        let mut buf: Vec<Vec<Vec<u8>>> = Vec::new();

        }
        let mut rows: Vec<Vec<Vec<u8>>> = Vec::new();
        let chunk_length: u8 = self.ihdr.color_type.channels();

        // 1 is added to account for filter method byte
        let row_length = 1 + (((f32::from(self.ihdr.bit_depth.as_u8()) /8f32) * self.ihdr.width as f32).ceil() as u32 * (u32::from(chunk_length)));
        println!("row length {}", row_length);
        let filtered_rows: Vec<Vec<u8>> = buffer.chunks(row_length as usize).map(Vec::from).collect::<Vec<Vec<u8>>>();

        println!("num of rows {}", filtered_rows.len());
        for (idx, row) in filtered_rows.iter().enumerate() {
            println!("{:?}", row);
            rows.push(match FilterType::from_u8(row[0])? {
                FilterType::None => row[1..].chunks(chunk_length as usize).map(Vec::from).collect(),
                FilterType::Sub => filter::sub(&row[1..], chunk_length, true),
                FilterType::Up => filter::up(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length, true),
                FilterType::Average => filter::average(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length),
                FilterType::Paeth => filter::paeth(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length, true),
            });
        }

        let row16: Vec<Vec<Vec<u16>>>;

        if self.ihdr.color_type == ColorType::Indexed {
            let palette = match &self.plte {
                Some(plte) => plte,
                // a PNG cannot have an indexed color type without the plte chunk
                _ => unreachable!(),
            };
            Ok(Bitmap::new(rows.iter().map(|x| x.iter().map(|y| palette[y[0]].to_vec()).collect()).collect())?)
        } else if self.ihdr.bit_depth == BitDepth::Sixteen {
            unimplemented!()
        } else {
            // convert Vec<Vec<Vec<u8>>> to Vec<Vec<Vec<u16>>>
            // TODO: Find a better solution than 3 nested `map`s
            row16 = rows.iter().map(|x| x.iter().map(|y| y.iter().map(|z| u16::from(*z)).collect()).collect()).collect();
            Ok(Bitmap::new(row16)?)
        }

    }

    pub fn dimensions(&self) -> [u32; 2] {
        [self.ihdr.width, self.ihdr.height]
    }
}

fn main() -> io::Result<()> {
    let png = PNG::from_path(&format!("pngs/{}.png", FILE_NAME))?;
    // let png = PNG::from_path(r"C:\Users\Connor\Downloads\PngSuite-2017jul19\oi9n2c16.png")?;
impl <'a> PNG {
    pub fn from_path<S>(file_path: S) -> Result<Self, PNGDecodingError>
        where S: Into<std::borrow::Cow<'a, str>> + std::convert::AsRef<std::path::Path>
    {
        let file_size: usize = fs::metadata(&file_path)?.len() as usize;
        PNGDecoder::read(BufReader::with_capacity(file_size, File::open(file_path)?))
    }
}


fn main() -> Result<(), PNGDecodingError> {
    println!("{:?}", png);
    let pixels = png.pixels()?;
    let mut f = File::create("fogkfkg.json")?;
    f.write(serde_json::to_string(&pixels)?.as_bytes())?;
    // println!("\n{:?}", pixels[0][0]);
    Ok(())
}
