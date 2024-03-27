use crate::errors::MetadataError;

/// The PNG header. In ascii, it can be represented as \x{89}PNG\r\n\x{1a}\n
pub const HEADER: [u8; 8] = [137u8, 80, 78, 71, 13, 10, 26, 10];

/// The IEND chunk. It always has a length of 0, and so it is always the same between PNGs
pub const IEND: [u8; 12] = [0u8, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130];

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2,
    Indexed = 3,
    GrayscaleAlpha = 4,
    #[default]
    RGBA = 6,
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

    /// Number of unique channels per pixel.
    ///
    /// For example, RGB has 3 channels (red, green, and blue), while grayscale
    /// has 1 channel
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
pub struct Bitmap {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub bpp: usize,
}

impl Bitmap {
    pub fn new(
        width: u32,
        height: u32,
        bpp: usize,
        buffer: Vec<u8>,
    ) -> Result<Bitmap, MetadataError> {
        // if rows.is_empty() || rows.len() > 2_usize.pow(31) {
        //     return Err(MetadataError::InvalidHeight { height: rows.len() });
        // }

        Ok(Bitmap {
            width,
            height,
            bpp,
            buffer,
        })
    }

    pub fn rows(&self) -> impl Iterator<Item = &[u8]> {
        self.buffer.chunks_exact(self.width as usize * self.bpp)
    }

    pub fn flip(&mut self) {
        todo!()
        // self.rows.reverse();
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
