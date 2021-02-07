use std::{
    convert::AsRef,
    fmt,
    fs::{self, File},
    io::{BufReader, Read},
    path::Path,
};

use flate2::bufread::ZlibDecoder;

use crate::{
    decoder::PngDecoder,
    errors::{ChunkError, PngDecodingError},
    filter::{self, FilterType},
    {
        chunks::{pHYs, AncillaryChunks, ICCProfile, Unit, UnrecognizedChunk, IHDR, PLTE},
        Channel, Pixel,
    },
    {
        common::{BitDepth, Bitmap, ColorType, DPI},
        Dimensions,
    },
};

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub struct Png {
    pub ihdr: IHDR,
    pub plte: Option<PLTE>,
    pub idat: Vec<u8>,
    pub unrecognized_chunks: Vec<UnrecognizedChunk>,
    pub ancillary_chunks: AncillaryChunks,
}

impl fmt::Debug for Png {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Png")
            .field("ihdr", &self.ihdr)
            .field("plte", &self.plte)
            .field("data", &format!("{} bytes (compressed)", self.idat.len()))
            .field("unrecognized_chunks", &self.unrecognized_chunks)
            .field("ancillary_chunks", &self.ancillary_chunks)
            .finish()
    }
}

impl Png {
    pub fn open(file_path: impl AsRef<Path>) -> Result<Self, PngDecodingError> {
        let file_size: usize = fs::metadata(&file_path)?.len() as usize;
        PngDecoder::read(BufReader::with_capacity(file_size, File::open(file_path)?))
    }

    pub fn pixels(&self) -> Result<Bitmap, PngDecodingError> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut zlib = ZlibDecoder::new(&self.idat as &[u8]);
        let buf_len = zlib.read_to_end(&mut buffer)?;
        if buf_len == 0 {
            return Err(PngDecodingError::ZeroLengthIDAT);
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

        fn get_pixel(bytes: &mut [u8], color_type: ColorType, bit_depth: BitDepth) -> Pixel {
            assert_eq!(
                bytes.len() * 8,
                color_type.channels() as usize * bit_depth as usize
            );

            let byte_offset = &mut 0;
            fn get_next_channel(
                bytes: &mut [u8],
                bit_depth: BitDepth,
                byte_offset: &mut usize,
            ) -> Channel {
                match bit_depth {
                    BitDepth::Eight => {
                        let channel = Channel::Eight(bytes[*byte_offset]);
                        *byte_offset += 1;
                        channel
                    }
                    _ => todo!(),
                }
            }

            match color_type {
                ColorType::Grayscale => {
                    Pixel::Grayscale(get_next_channel(bytes, bit_depth, byte_offset))
                }
                ColorType::GrayscaleAlpha => Pixel::GrayscaleAlpha(
                    get_next_channel(bytes, bit_depth, byte_offset),
                    get_next_channel(bytes, bit_depth, byte_offset),
                ),
                ColorType::Indexed => {
                    Pixel::Indexed(get_next_channel(bytes, bit_depth, byte_offset))
                }
                ColorType::RGB => Pixel::Rgb {
                    red: get_next_channel(bytes, bit_depth, byte_offset),
                    green: get_next_channel(bytes, bit_depth, byte_offset),
                    blue: get_next_channel(bytes, bit_depth, byte_offset),
                },
                ColorType::RGBA => Pixel::Rgba {
                    red: get_next_channel(bytes, bit_depth, byte_offset),
                    green: get_next_channel(bytes, bit_depth, byte_offset),
                    blue: get_next_channel(bytes, bit_depth, byte_offset),
                    alpha: get_next_channel(bytes, bit_depth, byte_offset),
                },
            }
        }

        // convert Vec<Vec<Vec<u8>>> to Vec<Vec<Pixel>>
        let rows: Vec<Vec<Pixel>> = rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|mut pixel| {
                        get_pixel(&mut pixel, self.ihdr.color_type, self.ihdr.bit_depth)
                    })
                    .collect()
            })
            .collect();

        Ok(Bitmap::new(rows)?)
    }

    pub const fn dimensions(&self) -> Dimensions {
        Dimensions {
            width: self.ihdr.width as usize,
            height: self.ihdr.height as usize,
        }
    }

    pub fn palette(&self) -> Result<&PLTE, ChunkError> {
        match self.plte.as_ref() {
            Some(x) => Ok(x),
            None => Err(ChunkError::PLTEChunkNotFound),
        }
    }

    pub fn iccp_profile(&self) -> Result<ICCProfile, PngDecodingError> {
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

#[allow(dead_code)]
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
