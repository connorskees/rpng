use std::fmt;
use crate::common::{BitDepth, ColorType, CompressionType, Unit};
use crate::filter::{FilterMethod};
use crate::interlacing::{Interlacing};

/// The IHDR chunk contains important metadata for reading the image
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: BitDepth,
    pub color_type: ColorType,
    pub compression_type: CompressionType,
    pub filter_method: FilterMethod,
    pub interlace_method: Interlacing,
}

impl IHDR {
    pub fn new(
            width: u32,
            height: u32, 
            bit_depth: BitDepth, 
            color_type: ColorType, 
            compression_type: CompressionType, 
            filter_method: FilterMethod, 
            interlace_method: Interlacing
        ) -> Result<Self, &'static str> {

        if !(0 < width && width < 2u32.pow(31)) {
            return Err("width not between 0 and 2^31");
        }
        
        if !(0 < height && height < 2u32.pow(31)) {
            return Err("height not between 0 and 2^31");
        }

        match color_type {
            ColorType::Grayscale => {
                if ![BitDepth::One, BitDepth::Two, BitDepth::Four, BitDepth::Eight, BitDepth::Sixteen].contains(&bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::RGB => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&bit_depth) {
                    return Err("invalid bit depth for color type"); 
                }
            },
            ColorType::Indexed => {
                if ![BitDepth::One, BitDepth::Two, BitDepth::Four, BitDepth::Eight].contains(&bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::GrayscaleAlpha => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::RGBA => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
        }
    Ok(Self {
        width, height, bit_depth, color_type, compression_type, filter_method, interlace_method
    })
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct UnrecognizedChunk {
    pub length: u32,
    pub chunk_type: String,
    pub bytes: std::vec::Vec<u8>,
    pub is_critical: bool,
    pub is_public: bool,
    pub is_safe_to_copy: bool,
}

impl fmt::Debug for UnrecognizedChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UnrecognizedChunk {{\n    length: {}\n    chunk_type: \"{}\"\n}}", self.length, self.chunk_type)
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PaletteEntry {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl PaletteEntry {
    pub fn from_u8(val: &[u8]) -> Self {
        Self {
            red: val[0],
            green: val[1],
            blue: val[2],
        }
    }
}

/// The PLTE chunk contains a list of palette entries
#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub struct PLTE {
    pub entries: Vec<PaletteEntry>
}

impl fmt::Debug for PLTE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PLTE {{ {} entries }}", self.entries.len())
    }
}

/// The pHYs chunk contains information about the aspect ratio
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct pHYs {
    pub pixels_per_unit_x: u32,
    pub pixels_per_unit_y: u32,
    pub unit: Unit,
}

/// The iTXt chunk contains utf8 text
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct iTXt {
    pub keyword: String,
    pub compressed: bool, // compression flag: 0=false; 1=true
    pub compression_method: Option<CompressionType>,
    pub language_tag: String,
    pub translated_keyword: String,
    pub text: String,
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct gAMA {
    pub gamma: u32
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct cHRM {
    pub white_point_x: u32,
    pub white_point_y: u32,
    pub red_x: u32,
    pub red_y: u32,
    pub green_x: u32,
    pub green_y: u32,
    pub blue_x: u32,
    pub blue_y: u32,
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct iCCP {
    pub profile_name: String,
    pub compression_method: CompressionType,
    pub compressed_profile: Vec<u8>,
}

/// Ancillary chunks are those that are not necessary to render the image
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AncillaryChunks {
    pub phys: Option<pHYs>,
    pub itxt: Vec<Option<iTXt>>,
    pub gama: Option<gAMA>,
    pub chrm: Option<cHRM>,
    pub iccp: Option<iCCP>,
}

impl AncillaryChunks {
    pub fn new() -> Self {
        Self {
            phys: None,
            itxt: Vec::new(),
            gama: None,
            chrm: None,
            iccp: None,
        }
    }
}
