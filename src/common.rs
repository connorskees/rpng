use std::ops::Index;

#[cfg(feature = "serialize")]
use serde::ser::{Serialize, SerializeSeq, Serializer};

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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Channel {
    One(bool),
    Two(u8),
    Four(u8),
    Eight(u8),
    Sixteen(u16),
}

impl Channel {
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::One(a) => vec![a as u8],
            Self::Two(a) => vec![a],
            Self::Four(a) => vec![a],
            Self::Eight(a) => vec![a],
            Self::Sixteen(a) => a.to_be_bytes().to_vec(),
        }
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            &Self::One(a) => serializer.serialize_u8(a as u8),
            &Self::Two(a) => serializer.serialize_u8(a as u8),
            &Self::Four(a) => serializer.serialize_u8(a as u8),
            &Self::Eight(a) => serializer.serialize_u8(a as u8),
            &Self::Sixteen(a) => serializer.serialize_u16(a),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Pixel {
    Grayscale(Channel),
    GrayscaleAlpha(Channel, Channel),
    Indexed(Channel),
    Rgb {
        red: Channel,
        green: Channel,
        blue: Channel,
    },
    Rgba {
        red: Channel,
        green: Channel,
        blue: Channel,
        alpha: Channel,
    },
}

impl Pixel {
    pub fn channels(&self) -> usize {
        match self {
            Self::Grayscale(..) => 1,
            Self::Rgb { .. } => 3,
            Self::Indexed(..) => 1,
            Self::GrayscaleAlpha(..) => 2,
            Self::Rgba { .. } => 4,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Grayscale(grayscale) => grayscale.into_bytes(),
            Self::Rgb { red, green, blue } => {
                vec![red.into_bytes(), green.into_bytes(), blue.into_bytes()]
                    .into_iter()
                    .flatten()
                    .collect()
            }
            Self::Indexed(idx) => idx.into_bytes(),
            Self::GrayscaleAlpha(grayscale, alpha) => {
                vec![grayscale.into_bytes(), alpha.into_bytes()]
                    .into_iter()
                    .flatten()
                    .collect()
            }
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => vec![
                red.into_bytes(),
                green.into_bytes(),
                blue.into_bytes(),
                alpha.into_bytes(),
            ]
            .into_iter()
            .flatten()
            .collect(),
        }
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Pixel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.channels()))?;

        match self {
            Self::Grayscale(grayscale) => seq.serialize_element(grayscale)?,
            Self::Rgb { red, green, blue } => {
                seq.serialize_element(red)?;
                seq.serialize_element(green)?;
                seq.serialize_element(blue)?;
            }
            Self::Indexed(idx) => seq.serialize_element(idx)?,
            Self::GrayscaleAlpha(grayscale, alpha) => {
                seq.serialize_element(grayscale)?;
                seq.serialize_element(alpha)?;
            }
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => {
                seq.serialize_element(red)?;
                seq.serialize_element(green)?;
                seq.serialize_element(blue)?;
                seq.serialize_element(alpha)?;
            }
        }

        seq.end()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Bitmap {
    pub rows: Vec<Vec<Pixel>>,
    dimensions: Dimensions,
}

impl Bitmap {
    pub fn new(rows: Vec<Vec<Pixel>>) -> Result<Bitmap, MetadataError> {
        if rows.is_empty() || rows.len() > 2_usize.pow(31) {
            return Err(MetadataError::InvalidHeight { height: rows.len() });
        }

        Ok(Bitmap {
            dimensions: Dimensions {
                width: rows[0].len(),
                height: rows.len(),
            },
            rows,
        })
    }

    pub const fn width(&self) -> usize {
        self.dimensions.width
    }

    pub const fn height(&self) -> usize {
        self.dimensions.height
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.rows[y][x] = pixel;
    }

    pub fn flip(&mut self) {
        self.rows.reverse();
    }

    /// Convert pixels as flat array of bytes
    pub fn into_buffer(self) -> Vec<u8> {
        self.rows
            .into_iter()
            .flat_map(|row| {
                row.into_iter()
                    .flat_map(|pixel| pixel.into_bytes())
                    .collect::<Vec<u8>>()
            })
            .collect()
    }
}

impl Index<[usize; 2]> for Bitmap {
    type Output = Pixel;

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
