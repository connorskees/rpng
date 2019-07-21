use crate::errors::MetadataError;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum BitDepth {
    /// Colors are represented by a single bit. Black or white
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum Unit {
    Unknown = 0,
    Meters = 1,
}

impl std::default::Default for Unit {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Unit {
    pub fn from_u8(unit: u8) -> Result<Self, MetadataError> {
        match unit {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Meters),
            _ => Err(MetadataError::UnrecognizedUnit{ unit }),
        }
    }
}

pub fn get_bit_at(num: u8, n: u8) -> Result<u8, ()> {
    if n < 32 {
        Ok(num & (1 << n))
    } else {
        Err(())
    }
}