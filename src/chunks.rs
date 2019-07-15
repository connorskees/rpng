#[allow(non_camel_case_types)]
use std::fmt;
use crate::common::{BitDepth, ColorType, CompressionType, Interlacing, Unit};
use crate::filter::{FilterMethod};

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

#[derive(Default)]
pub struct PLTE {
    pub entries: Vec<PaletteEntry>
}

impl fmt::Debug for PLTE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PLTE {{ {} entries }}", self.entries.len())
    }
}

#[derive(Default, Debug)]
pub struct pHYs {
    pub pixels_per_unit_x: u32,
    pub pixels_per_unit_y: u32,
    pub unit: Unit,
}

#[derive(Default, Debug)]
pub struct iTXt {
    pub keyword: String,
    pub compressed: bool, // compression flag: 0=false; 1=true
    pub compression_method: u8,
    pub language_tag: String,
    pub translated_keyword: String,
    pub text: String,
}

#[derive(Default, Debug)]
pub struct gAMA {
    pub gamma: u32
}

#[derive(Default, Debug)]
pub struct AncillaryChunks {
    pub phys: Option<pHYs>,
    pub itxt: Vec<Option<iTXt>>,
    pub gama: Option<gAMA>,
}

impl AncillaryChunks {
    pub fn new() -> Self {
        Self {
            phys: None,
            itxt: Vec::new(),
            gama: None,
        }
    }
}
