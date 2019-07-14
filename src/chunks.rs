#[allow(non_camel_case_types)]
use std::fmt;
use crate::common::{ColorType, Unit};

#[derive(Default, Debug)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_type: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

pub struct Chunk {
    pub length: u32,
    pub chunk_type: String,
    pub bytes: std::vec::Vec<u8>,
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk {{\n    length: {}\n    chunk_type: \"{}\"\n}}", self.length, self.chunk_type)
    }
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
                if ![1, 2, 4, 8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::RGB => {
                if ![8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type"); 
                }
            },
            ColorType::Indexed => {
                if ![8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::GrayscaleAlpha => {
                if ![8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            ColorType::RGBA => {
                if ![8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
        }

        if self.compression_type != 0 {
            return Err("unrecognized compression type");
        }

        if self.filter_method != 0 {
            return Err("unrecognized filter method");
        }

        if self.interlace_method != 0 && self.interlace_method != 1 {
            return Err("unrecognized interlace method");
        }        
    Ok(())
    }
}

#[derive(Default, Debug)]
pub struct PaletteEntries {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl PaletteEntries {
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
    pub entries: Vec<PaletteEntries>
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
pub struct AncillaryChunks {
    pub phys: Option<pHYs>,
    pub itxt: Vec<Option<iTXt>>,
}

impl AncillaryChunks {
    pub fn new() -> Self {
        Self {
            phys: None,
            itxt: Vec::new(),
        }
    }
}
