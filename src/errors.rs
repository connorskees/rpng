use std::{fmt, io};
use crate::common::{BitDepth, ColorType};

/// Container for errors that can occur when decoding a PNG
#[derive(Debug)]
pub enum PNGDecodingError {
    InvalidHeader{found: [u8; 8], expected: [u8; 8]},
    InvalidIHDRLength(u32),
    MetadataError(MetadataError),
    FilterError(FilterError),
    IoError(io::Error),
    ZeroLengthIDAT,
    StringDecodeError(std::str::Utf8Error),
    ChunkError(ChunkError),
}

/// Errors dealing with critical and ancillary chunks
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ChunkError {
    UnexpectedPLTEChunk,
    PLTEChunkNotFound,
    InvalidPLTELength,
    ICCPChunkNotFound,
    InvalidgAMALength,
    UnrecognizedCriticalChunk(String),
    UnrecognizedsRGBValue(u8),
}


impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChunkError::*;
        match self {
            UnrecognizedCriticalChunk(name) => {
                write!(f, "Found unrecognized critical chunk '{}'", name)
            },
            UnexpectedPLTEChunk => {
                write!(f, "Unexpected PLTE chunk found")
            },
            PLTEChunkNotFound => {
                write!(f, "No PLTE chunk was found")
            },
            InvalidPLTELength => {
                write!(f, "PLTE chunk length was not divisible by 3 (and so doesn't properly give RGB values)")
            },
            ICCPChunkNotFound => {
                write!(f, "An ICC profile was not found")
            },
            InvalidgAMALength => {
                write!(f, "gAMA chunk length was not equal to 4")
            },
            UnrecognizedsRGBValue(val) => {
                write!(f, "Found {}, but expected value in 0..=3", val)
            }
        }
    }
}

/// Errors dealing with data that describe the PNG file
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

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use MetadataError::*;
        match self {
            UnrecognizedBitDepth{ bit_depth } => {
                write!(f, "Expected bit depth in [1, 2, 4, 8, 16], but found {}", bit_depth)
            },
            UnrecognizedCompressionType{ compression_type } => {
                write!(f, "Expected compression type in [0, 1], but found {}", compression_type)
            },
            UnrecognizedUnit{ unit } => {
                write!(f, "Expected unit in [0, 1], but found {}", unit)
            },
            UnrecognizedColorType{ color_type } => {
                write!(f, "Expected color type in [0, 2, 3, 4, 6], but found {}", color_type)
            },
            UnrecognizedInterlacingType{ interlacing_type } => {
                write!(f, "Expected interlacing type in [0, 1], but found {}", interlacing_type)
            },
            InvalidWidth{ width } => {
                write!(f, "Expected width in 1..=2**31, but found {}", width)
            },
            InvalidHeight{ height } => {
                write!(f, "Expected height in 1..=2**31, but found {}", height)
            },
            InvalidBitDepthForColorType{ bit_depth, color_type } => {
                write!(f, "Found incompatible bit depth and color type combination: bit_depth: {:?} - color_type: {:?}", bit_depth, color_type)
            },
        }
    }
}

/// Errors related to filtering
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FilterError {
    UnrecognizedFilterMethod(u16),
    UnrecognizedFilterType(u8),
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FilterError::*;
        match self {
            UnrecognizedFilterMethod(val) => {
                write!(f, "Expected value in 0..=4, but found {}", val)
            },
            UnrecognizedFilterType(val) => {
                write!(f, "Expected value of 0, but found {}", val)
            },
        }
    }
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
            MetadataError(err) => {
                write!(f, "{}", err)
            },
            FilterError(err) => {
                write!(f, "{}", err)
            },
            IoError(err) => {
                write!(f, "{}", err)
            },
            ZeroLengthIDAT => {
                write!(f, "no pixel data provided")
            },
            StringDecodeError(err) => {
                write!(f, "{}", err)
            },
            ChunkError(err) => {
                write!(f, "{}", err)
            }
        }
    }

}

macro_rules! convert_to_decoding_error {
    ($val:ident) => (
        impl std::convert::From<$val> for PNGDecodingError {
            fn from(error: $val) -> Self {
                PNGDecodingError::$val(error)
            }
        }
    )
}

convert_to_decoding_error!(FilterError);
convert_to_decoding_error!(MetadataError);
convert_to_decoding_error!(ChunkError);

impl std::convert::From<io::Error> for PNGDecodingError {
    fn from(error: io::Error) -> Self {
        PNGDecodingError::IoError(error)
    }
}

impl std::convert::From<std::str::Utf8Error> for PNGDecodingError {
    fn from(error: std::str::Utf8Error) -> Self {
        PNGDecodingError::StringDecodeError(error)
    }
}