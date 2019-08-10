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

impl std::convert::Into<u8> for BitDepth {
    fn into(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
            Self::Eight => 8,
            Self::Sixteen => 16,
        }
    }
}

impl BitDepth {
    /// Map BitDepth to its integer representation.
    /// Returns Err(UnrecognizedBitDepth) on unknown depth
    pub fn from_u8(bit_depth: u8) -> Result<BitDepth, MetadataError>  {
        match bit_depth {
            1 =>  Ok(Self::One),
            2 =>  Ok(Self::Two),
            4 =>  Ok(Self::Four),
            8 =>  Ok(Self::Eight),
            16 => Ok(Self::Sixteen),
            _ => Err(MetadataError::UnrecognizedBitDepth{ bit_depth })
        }
    }

    /// Map BitDepth to its integer representation.
    pub fn as_u8(self) -> u8 {
        self.into()
    }
}

impl std::default::Default for BitDepth {
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
            _ => Err(MetadataError::UnrecognizedCompressionType{ compression_type })
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            CompressionType::Deflate => 0,
        }
    }
}

impl std::convert::Into<u8> for CompressionType {
    fn into(self) -> u8 {
        match self {
            CompressionType::Deflate => 0,
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
    /// Map ColorType to its integer representation.
    /// Returns Err(UnrecognizedColorType) on unknown type
    pub fn from_u8(color_type: u8) -> Result<Self, MetadataError> {
        match color_type {
            0 => Ok(ColorType::Grayscale),
            2 => Ok(ColorType::RGB),
            3 => Ok(ColorType::Indexed),
            4 => Ok(ColorType::GrayscaleAlpha),
            6 => Ok(ColorType::RGBA),
            _ => Err(MetadataError::UnrecognizedColorType{ color_type })
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            ColorType::Grayscale => 0,
            ColorType::RGB => 2,
            ColorType::Indexed => 3,
            ColorType::GrayscaleAlpha => 4,
            ColorType::RGBA => 6,
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
            return Err(MetadataError::InvalidHeight{ height: rows.len() });
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

#[allow(dead_code, unused_imports)]
mod test {
    use super::*;

    #[test]
    fn test_bit_depth_conversions() {
        assert_eq!(BitDepth::One, BitDepth::from_u8(1).unwrap());
        assert_eq!(BitDepth::Two, BitDepth::from_u8(2).unwrap());
        assert_eq!(BitDepth::Four, BitDepth::from_u8(4).unwrap());
        assert_eq!(BitDepth::Eight, BitDepth::from_u8(8).unwrap());
        assert_eq!(BitDepth::Sixteen, BitDepth::from_u8(16).unwrap());
        assert!(BitDepth::from_u8(17).is_err());
        
        assert_eq!(BitDepth::One.as_u8(), 1);
        assert_eq!(BitDepth::Two.as_u8(), 2);
        assert_eq!(BitDepth::Four.as_u8(), 4);
        assert_eq!(BitDepth::Eight.as_u8(), 8);
        assert_eq!(BitDepth::Sixteen.as_u8(), 16);
    }

    fn test_get_bit_at() {
        assert_eq!(true, get_bit_at(0b001, 0));
        assert_eq!(false, get_bit_at(0b001, 0));
        assert_eq!(false, get_bit_at(0b001, 0));
    }
}