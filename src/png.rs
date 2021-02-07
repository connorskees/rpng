use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::{convert::AsRef, vec};
use std::{fmt, fs};

use flate2::bufread::ZlibDecoder;

use crate::chunks::{pHYs, AncillaryChunks, ICCProfile, Unit, UnrecognizedChunk, IHDR, PLTE};
use crate::common::{get_bit_at, BitDepth, Bitmap, ColorType, DPI};
use crate::decoder::PNGDecoder;
use crate::errors::{ChunkError, PNGDecodingError};
use crate::filter::{self, FilterType};

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub struct PNG {
    pub ihdr: IHDR,
    pub plte: Option<PLTE>,
    pub idat: Vec<u8>,
    pub unrecognized_chunks: Vec<UnrecognizedChunk>,
    pub ancillary_chunks: AncillaryChunks,
}

impl fmt::Debug for PNG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PNG")
            .field("ihdr", &self.ihdr)
            .field("plte", &self.plte)
            .field("data", &format!("{} bytes (compressed)", self.idat.len()))
            .field("unrecognized_chunks", &self.unrecognized_chunks)
            .field("ancillary_chunks", &self.ancillary_chunks)
            .finish()
    }
}

impl PNG {
    pub fn open(file_path: impl AsRef<Path>) -> Result<Self, PNGDecodingError> {
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

        let mut rows: Vec<Vec<Vec<u8>>> = Vec::new();
        let chunk_length: u8 = self.ihdr.color_type.channels();

        // 1 is added to account for filter method byte
        let row_length = 1
            + (((f32::from(self.ihdr.bit_depth as u8) / 8_f32) * self.ihdr.width as f32).ceil()
                as u32
                * (u32::from(chunk_length)));

        let filtered_rows: Vec<Vec<u8>> = buffer
            .chunks(row_length as usize)
            .map(Vec::from)
            .collect::<Vec<Vec<u8>>>();

        for (idx, row) in filtered_rows.iter().enumerate() {
            rows.push(match FilterType::from_u8(row[0])? {
                FilterType::None => row[1..]
                    .chunks(chunk_length as usize)
                    .map(Vec::from)
                    .collect(),
                FilterType::Sub => filter::sub(&row[1..], chunk_length, true),
                FilterType::Up => filter::up(
                    &row[1..],
                    if idx == 0 { None } else { Some(&rows[idx - 1]) },
                    chunk_length,
                    true,
                ),
                FilterType::Average => filter::average(
                    &row[1..],
                    if idx == 0 { None } else { Some(&rows[idx - 1]) },
                    chunk_length,
                ),
                FilterType::Paeth => filter::paeth(
                    &row[1..],
                    if idx == 0 { None } else { Some(&rows[idx - 1]) },
                    chunk_length,
                    true,
                ),
            });
        }

        // convert Vec<Vec<Vec<u8>>> to Vec<Vec<Vec<u16>>>
        let row16: Vec<Vec<Vec<u16>>>;

        match self.ihdr.bit_depth {
            BitDepth::One => {
                row16 = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|pixel| {
                                pixel
                                    .iter()
                                    .map(|channel| {
                                        (0..=7)
                                            .map(move |a| vec![u16::from(get_bit_at(*channel, a))])
                                            .collect::<Vec<Vec<u16>>>()
                                    })
                                    .flatten()
                                    .collect::<Vec<Vec<u16>>>()
                            })
                            .flatten()
                            .collect()
                    })
                    .collect();
            }
            BitDepth::Two => todo!(),
            BitDepth::Four => todo!(),
            BitDepth::Eight => {
                // TODO: Find a better solution than 3 nested `map`s
                row16 = rows
                    .iter()
                    .map(|row| {
                        vec![row
                            .iter()
                            .map(|pixel| {
                                pixel
                                    .iter()
                                    .map(|channel| u16::from(*channel))
                                    .collect::<Vec<u16>>()
                            })
                            .collect()]
                    })
                    .flatten()
                    .collect();
            }
            BitDepth::Sixteen => row16 = combine_u8s_to_u16(rows),
        }

        if self.ihdr.color_type == ColorType::Indexed {
            let palette = match &self.plte {
                Some(plte) => plte,
                // a PNG cannot have an indexed color type without the plte chunk
                _ => unreachable!(),
            };
            return Ok(Bitmap::<u16>::new(
                row16
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|pixel| {
                                pixel
                                    .chunks(1)
                                    .map(|channel| palette[channel[0]].to_vec())
                                    .collect::<Vec<Vec<u16>>>()
                            })
                            .flatten()
                            .collect()
                    })
                    .collect(),
            )?);
        }

        Ok(Bitmap::new(row16)?)
    }

    pub const fn dimensions(&self) -> [u32; 2] {
        [self.ihdr.width, self.ihdr.height]
    }

    pub fn palette(&self) -> Result<&PLTE, ChunkError> {
        match self.plte.as_ref() {
            Some(x) => Ok(x),
            None => Err(ChunkError::PLTEChunkNotFound),
        }
    }

    pub fn iccp_profile(&self) -> Result<ICCProfile, PNGDecodingError> {
        let iccp = match self.ancillary_chunks.iCCP.as_ref() {
            Some(x) => x,
            None => return Err(ChunkError::PLTEChunkNotFound.into()),
        };
        let mut zlib = ZlibDecoder::new(iccp.compressed_profile.as_slice());
        let mut buffer: Vec<u8> = Vec::new();
        zlib.read_to_end(&mut buffer)?;

        todo!()
    }

    pub fn dpi(&self) -> Option<DPI> {
        let meters_to_inch = 0.0254;
        let phys: &pHYs = match self.ancillary_chunks.pHYs.as_ref() {
            Some(x) => x,
            None => return None,
        };
        if phys.unit == Unit::Unknown {
            return None;
        }
        // `as` used here because the conversion from pixels/meter => pixels/inch will always
        // decrease the value, so we can guarantee that the value will never be greater than u32::MAX
        let dpi_x: u32 = (f64::from(phys.pixels_per_unit_x) * meters_to_inch).round() as u32;
        let dpi_y = (f64::from(phys.pixels_per_unit_y) * meters_to_inch).round() as u32;

        Some(DPI { dpi_x, dpi_y })
    }

    pub fn aspect_ratio(&self) /*-> Option<_> */
    {
        todo!()
    }

    /// `bpp` is defined as the number of bytes per complete pixel, rounding up to 1
    pub fn bpp(&self) -> u8 {
        std::cmp::max(
            1,
            (self.ihdr.bit_depth as u8 / 8) * self.ihdr.color_type.channels(),
        )
    }
}

fn combine_u8s_to_u16(bitmap: Vec<Vec<Vec<u8>>>) -> Vec<Vec<Vec<u16>>> {
    let mut b16: Vec<Vec<Vec<u16>>> = bitmap
        .iter()
        .map(|row| {
            vec![row
                .iter()
                .map(|pixel| {
                    pixel
                        .iter()
                        .map(|channel| u16::from(*channel))
                        .collect::<Vec<u16>>()
                })
                .collect()]
        })
        .flatten()
        .collect();
    for row in b16.iter_mut() {
        for pixel in row.iter_mut() {
            if pixel.len() < 2 {
                continue;
            }
            for channel in (0..pixel.len()).step_by(2) {
                pixel[channel] += pixel[channel + 1];
            }
            pixel.pop();
        }
    }
    b16
}
