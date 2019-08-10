use std::io::{BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::convert::AsRef;
use std::{fmt, fs};
use std::vec::Vec;

use flate2::bufread::ZlibDecoder;

use crate::common::{get_bit_at, Bitmap, BitDepth, ColorType, DPI};
use crate::chunks::{IHDR, PLTE, pHYs, Unit, UnrecognizedChunk, AncillaryChunks, ICCProfile};
use crate::decoder::PNGDecoder;
use crate::errors::{PNGDecodingError, ChunkError};
use crate::filter::{self, FilterType};
use crate::interlacing::{Interlacing};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PNG {
    pub ihdr: IHDR,
    pub plte: Option<PLTE>,
    pub idat: Vec<u8>,
    pub unrecognized_chunks: Vec<UnrecognizedChunk>,
    pub ancillary_chunks: AncillaryChunks,
}

impl fmt::Debug for PNG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "PNG {{\n    ihdr: {:?}\n    plte: {:?}\n    data: {} bytes (compressed)\n    unrecognized_chunks: {:#?}\n    ancillary_chunks: {:#?}\n}}",
            self.ihdr, self.plte.as_ref(), self.idat.len(), self.unrecognized_chunks, self.ancillary_chunks
        )
    }
}

impl PNG {
    pub fn open<S: AsRef<Path>>(file_path: S) -> Result<Self, PNGDecodingError> {
        let file_size: usize = fs::metadata(&file_path)?.len() as usize;
        PNGDecoder::read(BufReader::with_capacity(file_size, File::open(file_path)?))
    }

    pub fn pixels(&self) -> Result<Bitmap<u16>, PNGDecodingError> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut zlib = ZlibDecoder::new(&self.idat as &[u8]);
        let buf_len = zlib.read_to_end(&mut buffer)?;
        if buf_len == 0 {
            return Err(PNGDecodingError::ZeroLengthIDAT);
        }

        println!("{:?}", buffer);
        println!("interlaced length {:?}", buffer.len());

        let buf: Vec<Vec<Vec<u8>>> = if self.ihdr.interlace_method == Interlacing::Adam7 {
            Interlacing::adam7(self.ihdr.width, self.ihdr.height, buffer.clone())
        } else {
            Vec::new()
        };

        println!("deinterlaced length {}", buf.len());
        // println!("deinterlaced {:?}", buf);

        // match self.ihdr.interlace_method {
        //     Interlacing::None => (),//idat,
        //     Interlacing::Adam7 => Interlacing::adam7(self.ihdr.width, self.ihdr.height, buffer.clone())
        // };

        let mut rows: Vec<Vec<Vec<u8>>> = Vec::new();
        let chunk_length: u8 = self.ihdr.color_type.channels();

        // 1 is added to account for filter method byte
        let row_length = 1 + (((f32::from(self.ihdr.bit_depth.as_u8()) /8f32) * self.ihdr.width as f32).ceil() as u32 * (u32::from(chunk_length)));
        println!("row length {}", row_length);

        // println!("raw buf len {:?}", buffer.len());
        // println!("raw buf{:?}", buffer);

        let filtered_rows: Vec<Vec<u8>> = buffer.chunks(row_length as usize).map(Vec::from).collect::<Vec<Vec<u8>>>();

        println!("num of rows {}", filtered_rows.len());
        // println!("filtered_rows {:?}", filtered_rows);
        for (idx, row) in filtered_rows.iter().enumerate() {
            // println!("-{:?}", row);
            rows.push(match FilterType::from_u8(row[0])? {
                FilterType::None => row[1..].chunks(chunk_length as usize).map(Vec::from).collect(),
                FilterType::Sub => filter::sub(&row[1..], chunk_length, true),
                FilterType::Up => filter::up(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length, true),
                FilterType::Average => filter::average(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length),
                FilterType::Paeth => filter::paeth(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length, true),
            });
        }

        // convert Vec<Vec<Vec<u8>>> to Vec<Vec<Vec<u16>>>
        let row16: Vec<Vec<Vec<u16>>>;

        

        // println!("{:?}", rows);

        match self.ihdr.bit_depth {
            BitDepth::One => {
                row16 = rows
                        .iter()
                        .map(
                            |row| row.iter().map(
                                |pixel| pixel.iter().map(
                                    |channel| (0..=7).map(move |a| vec!(u16::from(get_bit_at(*channel, a)))).collect::<Vec<Vec<u16>>>()        
                                ).flatten().collect::<Vec<Vec<u16>>>()
                            ).flatten().collect()
                        ).collect();
            },
            BitDepth::Two => unimplemented!(),
            BitDepth::Four => unimplemented!(),
            BitDepth::Eight => {
                // TODO: Find a better solution than 3 nested `map`s
                row16 = rows
                        .iter()
                        .map(
                            |row| vec!(row.iter().map(
                                |pixel| pixel.iter().map(
                                    |channel| u16::from(*channel)
                                ).collect::<Vec<u16>>()
                            ).collect()
                            )
                        ).flatten().collect();
            },
            BitDepth::Sixteen => { row16 = combine_u8s_to_u16(rows) }
        }

        // println!("{:?}", row16);

        if self.ihdr.color_type == ColorType::Indexed {
            let palette = match &self.plte {
                Some(plte) => plte,
                // a PNG cannot have an indexed color type without the plte chunk
                _ => unreachable!(),
            };
            return Ok(Bitmap::<u16>::new(
                        row16
                        .iter()
                        .map(
                            |row| row.iter().map(
                                |pixel| pixel.chunks(1).map(
                                    |channel| palette[channel[0]].to_vec()
                                ).collect::<Vec<Vec<u16>>>()
                            ).flatten().collect()
                        ).collect())?
                    );
        }

        Ok(Bitmap::new(row16)?)
    }

    pub const fn dimensions(&self) -> [u32; 2] {
        [self.ihdr.width, self.ihdr.height]
    }

    pub fn palette(&self) -> Result<&PLTE, ChunkError> {
        match self.plte.as_ref() {
            Some(x) => Ok(x),
            None => Err(ChunkError::PLTEChunkNotFound)
        }
    }

    pub fn iccp_profile(&self) -> Result<ICCProfile, PNGDecodingError> {
        // let png = PNG::open(r"C:\Users\Connor\Documents\Fonts\Merry Christmas\merry-christmas_flag.png")?;
        let iccp = match self.ancillary_chunks.iCCP.as_ref() {
            Some(x) => x,
            None => return Err(ChunkError::PLTEChunkNotFound.into())
        };
        let mut zlib = ZlibDecoder::new(iccp.compressed_profile.as_slice());
        let mut buffer: Vec<u8> = Vec::new();
        zlib.read_to_end(&mut buffer)?;
        println!("{:?}", buffer.len());
        unimplemented!()
    }

    pub fn dpi(&self) -> Option<DPI> {
        let meters_to_inch = 0.0254;
        let phys: &pHYs = match self.ancillary_chunks.pHYs.as_ref() {
            Some(x) => x,
            None => return None
        };
        if phys.unit == Unit::Unknown {
            return None;
        }
        // `as` used here because the conversion from pixels/meter => pixels/inch will always
        // decrease the value, so we can guarantee that the value will never be greater than u32::MAX
        let dpi_x: u32 = (f64::from(phys.pixels_per_unit_x) * meters_to_inch).round() as u32;
        let dpi_y = (f64::from(phys.pixels_per_unit_y) * meters_to_inch).round() as u32;
        Some(DPI{ dpi_x, dpi_y })
    }

    pub fn aspect_ratio(&self) /*-> Option<_> */ {
        unimplemented!()
    }

    /// `bpp` is defined as the number of bytes per complete pixel, rounding up to 1
    pub fn bpp(&self) -> u8 {
        std::cmp::max(1, (self.ihdr.bit_depth.as_u8()/8) * self.ihdr.color_type.channels())
    }
}

fn combine_u8s_to_u16(bitmap: Vec<Vec<Vec<u8>>>) -> Vec<Vec<Vec<u16>>> {
    let mut b16: Vec<Vec<Vec<u16>>> = bitmap
                                    .iter()
                                    .map(
                                        |row| vec!(row.iter().map(
                                            |pixel| pixel.iter().map(
                                                |channel| u16::from(*channel)
                                            ).collect::<Vec<u16>>()
                                        ).collect()
                                        )
                                    ).flatten().collect();
    for row in b16.iter_mut() {
        for pixel in row.iter_mut() {
            if pixel.len() < 2 {
                continue
            }
            for channel in (0..pixel.len()).step_by(2) {
                pixel[channel] += pixel[channel+1];
            }
            pixel.pop();
        }
    }
    b16
}