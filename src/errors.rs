use std::{fmt, io};

use crate::common::{BitDepth, ColorType};

/// Container for errors that can occur when decoding a PNG
#[derive(Debug)]
pub enum PngDecodingError {
    /// The 8 byte PNG header was found to be incorrect, which indicates either an error in transmission
    ///  or that the user is attempting to parse something other than a PNG file
    InvalidHeader {
        found: [u8; 8],
        expected: [u8; 8],
    },
    InvalidIENDChunk {
        found: (u32, [u8; 4]),
        expected: [u8; 12],
    },
    /// IHDR length was found to be more or less than 13, which indicates a serious error
    InvalidIHDRLength(u32),
    MetadataError(MetadataError),
    FilterError(FilterError),
    IoError(io::Error),
    ZeroLengthIDAT,
    Utf8Error(std::str::Utf8Error),
    StringDecodeError(std::string::FromUtf8Error),
    ChunkError(ChunkError),
}

/// Errors dealing with critical and ancillary chunks
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ChunkError {
    /// A PLTE chunk was found in a color type other than indexed, RBA, or RGBA
    UnexpectedPLTEChunk,
    /// A PLTE chunk was not found in an indexed color type context
    PLTEChunkNotFound,
    /// The length of the PLTE chunk found did not fit `len % 3 == 0`, so is potentially corrupted
    InvalidPLTELength,
    /// Attempted to access ICC profile; however, no ICCP chunk was found
    ICCPChunkNotFound,
    /// The length of the gAMA chunk is known to be 4 bytes; however, this was not found
    InvalidgAMALength,
    /// A critical chunk (specified by a capital first letter) was not recognized
    UnrecognizedCriticalChunk(String),
    /// An sRGB value outside the range `0..=3` was found
    UnrecognizedsRGBValue(u8),
}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChunkError::*;
        match self {
            UnrecognizedCriticalChunk(name) => {
                write!(f, "found unrecognized critical chunk '{}'", name)
            }
            UnexpectedPLTEChunk => {
                write!(f, "unexpected PLTE chunk found")
            }
            PLTEChunkNotFound => {
                write!(f, "no PLTE chunk was found")
            }
            InvalidPLTELength => {
                write!(f, "PLTE chunk length was not divisible by 3 (and so doesn't properly give RGB values)")
            }
            ICCPChunkNotFound => {
                write!(f, "an ICC profile was not found")
            }
            InvalidgAMALength => {
                write!(f, "gAMA chunk length was not equal to 4")
            }
            UnrecognizedsRGBValue(val) => {
                write!(f, "found {}, but expected value in 0..=3", val)
            }
        }
    }
}

/// Errors dealing with data that describe the PNG file
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MetadataError {
    /// Bit depth was not in `[1, 2, 4, 8, 16]`
    UnrecognizedBitDepth { bit_depth: u8 },
    /// Compression type was not in `[0, 1]`
    UnrecognizedCompressionType { compression_type: u8 },
    /// Unit (from pHYs chunk) was not in `[0, 1]`
    UnrecognizedUnit { unit: u8 },
    /// Color type was not in `[0, 2, 3, 4, 6]`
    UnrecognizedColorType { color_type: u8 },
    /// Interlacing type was not in `[0, 1]`
    UnrecognizedInterlacingType { interlacing_type: u8 },
    /// Width was not in range `1..=2**31`
    InvalidWidth { width: usize },
    /// Height was not in range `1..=2**31`
    InvalidHeight { height: usize },
    /// An invalid bit depth and color type combination was found
    InvalidBitDepthForColorType {
        bit_depth: BitDepth,
        color_type: ColorType,
    },
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use MetadataError::*;
        match self {
            UnrecognizedBitDepth { bit_depth } => {
                write!(
                    f,
                    "expected bit depth in [1, 2, 4, 8, 16], but found {}",
                    bit_depth
                )
            }
            UnrecognizedCompressionType { compression_type } => {
                write!(
                    f,
                    "expected compression type in [0, 1], but found {}",
                    compression_type
                )
            }
            UnrecognizedUnit { unit } => {
                write!(f, "expected unit in [0, 1], but found {}", unit)
            }
            UnrecognizedColorType { color_type } => {
                write!(
                    f,
                    "expected color type in [0, 2, 3, 4, 6], but found {}",
                    color_type
                )
            }
            UnrecognizedInterlacingType { interlacing_type } => {
                write!(
                    f,
                    "expected interlacing type in [0, 1], but found {}",
                    interlacing_type
                )
            }
            InvalidWidth { width } => {
                write!(f, "expected width in 1..=2**31, but found {}", width)
            }
            InvalidHeight { height } => {
                write!(f, "expected height in 1..=2**31, but found {}", height)
            }
            InvalidBitDepthForColorType {
                bit_depth,
                color_type,
            } => {
                write!(f, "found incompatible bit depth and color type combination: bit_depth: {:?} - color_type: {:?}", bit_depth, color_type)
            }
        }
    }
}

/// Errors related to filtering
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
// TODO: consolidate with metadata error
pub enum FilterError {
    UnrecognizedFilterMethod(u16),
    UnrecognizedFilterType(u8),
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FilterError::*;
        match self {
            UnrecognizedFilterMethod(val) => {
                write!(f, "expected value in 0..=4, but found {}", val)
            }
            UnrecognizedFilterType(val) => {
                write!(f, "expected value of 0, but found {}", val)
            }
        }
    }
}

impl fmt::Display for PngDecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PngDecodingError::*;
        match self {
            InvalidHeader { found, expected } => {
                write!(f, "expected bytes {:?}, but found {:?}", expected, found)
            }
            InvalidIENDChunk { found, expected } => {
                write!(f, "expected bytes {:?}, but found {:?}", expected, found)
            }
            InvalidIHDRLength(len) => {
                write!(f, "expected 13, but found {}", len)
            }
            MetadataError(err) => {
                write!(f, "{}", err)
            }
            FilterError(err) => {
                write!(f, "{}", err)
            }
            IoError(err) => {
                write!(f, "{}", err)
            }
            ZeroLengthIDAT => {
                write!(f, "no pixel data provided")
            }
            StringDecodeError(err) => {
                write!(f, "{}", err)
            }
            Utf8Error(err) => {
                write!(f, "{}", err)
            }
            ChunkError(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

macro_rules! convert_to_decoding_error {
    ($val:ident) => {
        impl std::convert::From<$val> for PngDecodingError {
            fn from(error: $val) -> Self {
                PngDecodingError::$val(error)
            }
        }
    };
    ($name:ident, $original:ty) => {
        impl std::convert::From<$original> for PngDecodingError {
            fn from(error: $original) -> Self {
                PngDecodingError::$name(error)
            }
        }
    };
}

convert_to_decoding_error!(FilterError);
convert_to_decoding_error!(MetadataError);
convert_to_decoding_error!(ChunkError);
convert_to_decoding_error!(IoError, io::Error);
convert_to_decoding_error!(Utf8Error, std::str::Utf8Error);
convert_to_decoding_error!(StringDecodeError, std::string::FromUtf8Error);

#[derive(Debug)]
pub enum PngEncodingError {}
