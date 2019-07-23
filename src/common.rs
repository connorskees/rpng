use std::ops::Index;

use crate::errors::MetadataError;

/// Number of bits per color channel
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum BitDepth {
    /// Colors are represented by a single bit. Black or white
    One = 1,
    /// Color channels can be 0-3
    Two = 2,
    /// Color channels can be 0-15
    Four = 4,
    /// Color channels can be 0-255
    Eight = 8,
    /// Color channels can be 0-65_535
    Sixteen = 16,
}

impl BitDepth {
    pub fn from_u8(bit_depth: u8) -> Result<Self, MetadataError> {
        match bit_depth {
            1 =>  Ok(Self::One),
            2 =>  Ok(Self::Two),
            4 =>  Ok(Self::Four),
            8 =>  Ok(Self::Eight),
            16 => Ok(Self::Sixteen),
            _ => Err(MetadataError::UnrecognizedBitDepth{ bit_depth })
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
            Self::Eight => 8,
            Self::Sixteen => 16,
        }
    }
}

impl std::default::Default for BitDepth {
    fn default() -> Self {
        Self::Eight
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionType {
    /// zlib DEFLATE compression
    Deflate = 0,
}

impl CompressionType {
    pub fn from_u8(compression_type: u8) -> Result<Self, MetadataError> {
        match compression_type {
            0 => Ok(Self::Deflate),
            _ => Err(MetadataError::UnrecognizedCompressionType{ compression_type })
        }
    }
}

impl std::default::Default for CompressionType {
    fn default() -> Self {
        Self::Deflate
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2, // Truecolor
    Indexed = 3,
    GrayscaleAlpha = 4,
    RGBA = 6, // TruecolorAlpha
}

impl std::default::Default for ColorType {
    fn default() -> Self {
        ColorType::RGBA
    }
}

impl ColorType {
    pub fn from_u8(color_type: u8) -> Result<Self, MetadataError> {
        match color_type {
            0 => Ok(Self::Grayscale),
            2 => Ok(Self::RGB),
            3 => Ok(Self::Indexed),
            4 => Ok(Self::GrayscaleAlpha),
            6 => Ok(Self::RGBA),
            _ => Err(MetadataError::UnrecognizedColorType{ color_type })
        }
    }

    pub fn channels(self) -> u8 {
        match self {
            Self::Grayscale => 1,
            Self::RGB => 3,
            Self::Indexed => 1,
            Self::GrayscaleAlpha => 2,
            Self::RGBA => 4,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Bitmap<T> {
    pub rows: Vec<Vec<Vec<T>>>,
    width: usize,
    height: usize,
}

impl<T> Bitmap<T> {
    pub fn new(rows: Vec<Vec<Vec<T>>>) -> Result<Bitmap<T>, MetadataError> {
        if rows.is_empty() || rows.len() > 2usize.pow(31) {
            return Err(MetadataError::InvalidHeight{ height: rows.len() });
        }
        Ok(Bitmap {
            width: rows[0].len(),
            height: rows.len(),
            rows,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Vec<T>) {
        self.rows[y][x] = pixel;
    }
}

impl<T> Index<[usize; 2]> for Bitmap<T> {
    type Output = Vec<T>;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.rows[index[1]][index[0]]
    }
}

/// Get bit of big endian `num` at position `n`
pub fn get_bit_at(num: u8, n: u8) -> bool {
    (num & (1 << n)) != 0
}