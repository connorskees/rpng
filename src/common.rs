use std::ops::Index;

use crate::errors::MetadataError;

/// The PNG header. In ascii, it can be represented as \x{89}PNG\r\n\x{1a}\n
pub const HEADER: [u8; 8] = [137u8, 80, 78, 71, 13, 10, 26, 10];
/// The IEND chunk. It always has a length of 0, and so it is always the same between PNGs
pub const IEND: [u8; 12] = [0u8, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130];

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
    pub fn from_u8(bit_depth: u8) -> Result<BitDepth, MetadataError> {
        match bit_depth {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            4 => Ok(Self::Four),
            8 => Ok(Self::Eight),
            16 => Ok(Self::Sixteen),
            _ => Err(MetadataError::UnrecognizedBitDepth { bit_depth }),
        }
    }
}

impl Default for BitDepth {
    fn default() -> Self {
        Self::Eight
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
/// Compression type used on IDAT chunks
/// Currently, the only specified compression type is DEFLATE
pub enum CompressionType {
    /// zlib DEFLATE compression
    Deflate = 0,
}

impl CompressionType {
    pub fn from_u8(compression_type: u8) -> Result<CompressionType, MetadataError> {
        match compression_type {
            0 => Ok(CompressionType::Deflate),
            _ => Err(MetadataError::UnrecognizedCompressionType { compression_type }),
        }
    }
}

impl Default for CompressionType {
    fn default() -> Self {
        Self::Deflate
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2,
    Indexed = 3,
    GrayscaleAlpha = 4,
    RGBA = 6,
}

impl Default for ColorType {
    fn default() -> Self {
        ColorType::RGBA
    }
}

impl ColorType {
    /// Map ColorType to its integer representation.
    /// Returns Err(UnrecognizedColorType) on unknown type
    pub fn from_u8(color_type: u8) -> Result<Self, MetadataError> {
        match color_type {
            0 => Ok(ColorType::Grayscale),
            2 => Ok(ColorType::RGB),
            3 => Ok(ColorType::Indexed),
            4 => Ok(ColorType::GrayscaleAlpha),
            6 => Ok(ColorType::RGBA),
            _ => Err(MetadataError::UnrecognizedColorType { color_type }),
        }
    }

    /// Number of unique channels per pixel: for example, RGB has 3 channels (red, green, and blue); while
    /// grayscale has 1 channel (grayscale)
    pub fn channels(self) -> u8 {
        match self {
            ColorType::Grayscale => 1,
            ColorType::RGB => 3,
            ColorType::Indexed => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGBA => 4,
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
            return Err(MetadataError::InvalidHeight { height: rows.len() });
        }

        Ok(Bitmap {
            width: rows[0].len(),
            height: rows.len(),
            rows,
        })
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
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
/// where n is 0 indexed
///
/// # Examples
/// ```
/// use rpng::get_bit_at;
///
/// let number = 0b1101;
/// assert_eq!(1, u8::from(get_bit_at(number, 0)));
/// assert_eq!(0, u8::from(get_bit_at(number, 1)));
/// assert_eq!(1, u8::from(get_bit_at(number, 2)));
/// assert_eq!(1, u8::from(get_bit_at(number, 3)));
/// ```
pub const fn get_bit_at(num: u8, n: u8) -> bool {
    (num & (1 << n)) != 0
}

#[derive(Debug)]
pub struct DPI {
    pub dpi_x: u32,
    pub dpi_y: u32,
}
