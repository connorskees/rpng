use std::{fmt, io};
use crate::common::{BitDepth, ColorType};

#[derive(Debug)]
pub enum PNGDecodingError {
    InvalidHeader{found: [u8; 8], expected: [u8; 8]},
    InvalidIHDRLength(u32),
    UnrecognizedCriticalChunk(String),
    MetadataError(MetadataError),
    FilterError(FilterError),
    UnexpectedPLTEChunk,
    InvalidPLTELength,
    InvalidgAMALength,
    IoError(io::Error),
    ZeroLengthIDAT(&'static str),
    StringDecodeError(std::str::Utf8Error),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MetadataError {
    UnrecognizedBitDepth{ bit_depth: u8 },
    UnrecognizedCompressionType{ compression_type: u8 },
    UnrecognizedUnit{ unit: u8 },
    UnrecognizedColorType{ color_type: u8 },
    UnrecognizedInterlacingType{ interlacing_type: u8 },
    InvalidWidth{ width: usize },
    InvalidHeight{ height: usize },
    InvalidBitDepthForColorType{ bit_depth: BitDepth, color_type: ColorType }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FilterError {
    UnrecognizedFilterMethod(u16),
    UnrecognizedFilterType(u8),
}

impl fmt::Display for PNGDecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PNGDecodingError::*;
        match self {
            InvalidHeader{found, expected} => {
                write!(f, "Expected bytes {:?}, but found {:?}", expected, found)
            },
            InvalidIHDRLength(len) => {
                write!(f, "Expected 13, but found {}", len)
            },
            UnrecognizedCriticalChunk(name) => {
                write!(f, "Found unrecognized critical chunk '{}'", name)
            },

impl std::convert::From<io::Error> for PNGDecodingError {
    fn from(error: io::Error) -> Self {
        PNGDecodingError::IoError(error)
    }
}

impl std::convert::From<FilterError> for PNGDecodingError {
    fn from(error: FilterError) -> Self {
        PNGDecodingError::FilterError(error)
    }
}

impl std::convert::From<MetadataError> for PNGDecodingError {
    fn from(error: MetadataError) -> Self {
        PNGDecodingError::MetadataError(error)
    }
}

impl std::convert::From<std::str::Utf8Error> for PNGDecodingError {
    fn from(error: std::str::Utf8Error) -> Self {
        PNGDecodingError::StringDecodeError(error)
    }
}