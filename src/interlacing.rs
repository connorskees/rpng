use crate::errors::MetadataError;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum Interlacing {
    None = 0,
    Adam7 = 1,
}

impl std::default::Default for Interlacing {
    fn default() -> Self {
        Self::None
    }
}

impl Interlacing {
    pub fn from_u8(interlacing_type: u8) -> Result<Self, MetadataError> {
        match interlacing_type {
            0 => Ok(Self::None),
            1 => todo!("adam7 interlacing is not currently supported"),
            _ => Err(MetadataError::UnrecognizedInterlacingType { interlacing_type }),
        }
    }
}
