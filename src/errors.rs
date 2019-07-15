use std::io;

#[derive(Debug)]
pub enum PNGDecodingError {
    InvalidHeader([u8; 8], &'static str),
    InvalidIHDRLength(u32),
    UnrecognizedCriticalChunk,
    MetadataError(MetadataError),
    FilterError(FilterError),
    UnexpectedPLTEChunk,
    InvalidPLTELength,
    InvalidgAMALength,
    IoError(io::Error),
    ZeroLengthIDAT(&'static str),
}

#[derive(Debug)]
pub enum MetadataError {
    UnrecognizedBitDepth{ bit_depth: u8 },
    UnrecognizedCompressionType{ compression_type: u8 },
    UnrecognizedUnit{ unit: u8 },
    UnrecognizedColorType{ color_type: u8 },
    UnrecognizedInterlacingType{ interlacing_type: u8 },
}

#[derive(Debug)]
pub enum FilterError {
    UnrecognizedFilterMethod(u8),
    UnrecognizedFilterType(u8),
}

#[derive(Debug)]
pub enum ChunkError {}

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