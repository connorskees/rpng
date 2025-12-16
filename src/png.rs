use std::{
    fmt,
    fs::{self, File},
    io::{BufReader, Read},
    path::Path,
};

use flate2::bufread::ZlibDecoder;

use crate::{
    chunks::{pHYs, AncillaryChunks, ICCProfile, Unit, UnrecognizedChunk, IHDR, PLTE},
    common::{Bitmap, ColorType, DPI},
    decoder::PngDecoder,
    errors::{ChunkError, PngDecodingError},
    filter,
};

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub struct Png {
    pub ihdr: IHDR,
    pub plte: Option<PLTE>,
    pub idat: Vec<u8>,
    /// If present, the raw image data. No decoding necessary
    pub decoded_buffer: Option<Vec<u8>>,
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

    pub fn decode(&self) -> Bitmap {
        let mut decompressed_buffer = Vec::new();
        let mut zlib = ZlibDecoder::new(&self.idat as &[u8]);
        let buf_len = zlib.read_to_end(&mut decompressed_buffer).unwrap();
        assert_ne!(buf_len, 0, "zero length idat");

        let width = self.ihdr.width as usize;
        let height = self.ihdr.height as usize;

        let mut decoded_buffer = vec![0; decompressed_buffer.len() - height];

        let bytes_per_row = 1 + width * self.bpp();

        for i in 0..height {
            let raw_row_start = i * bytes_per_row;
            let decoded_row_start = raw_row_start - i;
            let start = decompressed_buffer[raw_row_start];
            let raw_row =
                &decompressed_buffer[(raw_row_start + 1)..(raw_row_start + bytes_per_row)];

            let (prev, decoded_row) = decoded_buffer.split_at_mut(decoded_row_start);

            let decoded_row = &mut decoded_row[..(bytes_per_row - 1)];

            let prev = &prev[(prev.len().saturating_sub(bytes_per_row - 1))..];

            if i != 0 {
                debug_assert_eq!(prev.len(), raw_row.len());
                debug_assert_eq!(prev.len(), decoded_row.len());
            }

            match start {
                0 => decoded_row[..raw_row.len()].copy_from_slice(&raw_row),
                1 => filter::sub(raw_row, decoded_row, self.bpp()),
                2 => filter::up(prev, raw_row, decoded_row),
                3 => filter::average(prev, raw_row, decoded_row, self.bpp()),
                4 => filter::paeth(prev, raw_row, decoded_row, self.bpp()),
                _ => unimplemented!("{}", start),
            }
        }

        Bitmap {
            width: self.ihdr.width,
            height: self.ihdr.height,
            bpp: self.bpp(),
            buffer: decoded_buffer,
        }
    }

    pub const fn dimensions(&self) -> (u32, u32) {
        (self.ihdr.width, self.ihdr.height)
    }

    pub const fn width(&self) -> u32 {
        self.ihdr.width
    }

    pub const fn height(&self) -> u32 {
        self.ihdr.height
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
    pub fn bpp(&self) -> usize {
        std::cmp::max(
            1,
            ((self.ihdr.bit_depth / 8) * self.ihdr.color_type.channels()) as usize,
        )
    }
}

#[derive(Debug)]
pub struct PngBuilder {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    interlaced: bool,
    color_type: ColorType,
    bit_depth: u8,
}

impl PngBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        PngBuilder {
            width,
            height,
            buffer: Vec::new(),
            interlaced: false,
            color_type: ColorType::RGBA,
            bit_depth: 8,
        }
    }

    pub fn interlaced(mut self, interlaced: bool) -> Self {
        self.interlaced = interlaced;
        self
    }

    pub fn color_type(mut self, color_type: ColorType) -> Self {
        self.color_type = color_type;
        self
    }

    pub fn buffer(mut self, buffer: Vec<u8>) -> Self {
        self.buffer = buffer;
        self
    }

    pub fn finish(self) -> Png {
        Png {
            ihdr: IHDR {
                width: self.width,
                height: self.height,
                bit_depth: self.bit_depth,
                color_type: self.color_type,
                compression_type: 0,
                filter_method: 0,
                interlace_method: 0,
            },
            plte: None,
            decoded_buffer: Some(self.buffer.clone()),
            idat: self.buffer,
            unrecognized_chunks: Vec::new(),
            ancillary_chunks: AncillaryChunks::new(),
        }
    }
}
