use std::fmt;
use crate::common::{BitDepth, ColorType, CompressionType, Interlacing, Unit};
use crate::filter::{FilterMethod};

/// The IHDR chunk contains important metadata for reading the image
#[derive(Default, Debug)]
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
    pub fn validate_fields(&self) -> Result<(), &'static str> {
        if !(0 < self.width && self.width < 2u32.pow(31)) {
            return Err("width not between 0 and 2^31");
        }
        
        if !(0 < self.height && self.height < 2u32.pow(31)) {
            return Err("height not between 0 and 2^31");
        }

        match self.color_type {
            ColorType::Grayscale => {
                if ![BitDepth::One, BitDepth::Two, BitDepth::Four, BitDepth::Eight, BitDepth::Sixteen].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::RGB => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type"); 
                }
            },
            ColorType::Indexed => {
                if ![BitDepth::One, BitDepth::Two, BitDepth::Four, BitDepth::Eight].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::GrayscaleAlpha => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::RGBA => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
        }
    Ok(())
    }
}

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

#[derive(Default, Debug)]
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
#[derive(Default)]
pub struct PLTE {
    pub entries: Vec<PaletteEntry>
}

impl fmt::Debug for PLTE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PLTE {{ {} entries }}", self.entries.len())
    }
}

/// The pHYs chunk contains information about the aspect ratio
#[derive(Default, Debug)]
#[allow(non_camel_case_types)]
pub struct pHYs {
    pub pixels_per_unit_x: u32,
    pub pixels_per_unit_y: u32,
    pub unit: Unit,
}

/// The iTXt chunk contains utf8 text
#[derive(Default, Debug)]
#[allow(non_camel_case_types)]
pub struct iTXt {
    pub keyword: String,
    pub compressed: bool, // compression flag: 0=false; 1=true
    pub compression_method: Option<CompressionType>,
    pub language_tag: String,
    pub translated_keyword: String,
    pub text: String,
}

#[derive(Default, Debug)]
#[allow(non_camel_case_types)]
pub struct gAMA {
    pub gamma: u32
}

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
#[allow(non_camel_case_types)]
pub struct iCCP {
    pub profile_name: String,
    pub compression_method: CompressionType,
    pub compressed_profile: Vec<u8>,
}

/// Ancillary chunks are those that are not necessary to render the image
#[derive(Default, Debug)]
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
