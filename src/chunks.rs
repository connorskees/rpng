use std::fmt;

#[derive(Default, Debug)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
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
            0 => {
                if ![1, 2, 4, 8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            2 => {
                if ![8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type"); 
                }
            },
            3 => {
                if ![8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            4 => {
                if ![8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            6 => {
                if ![8, 16].contains(&self.bit_depth) {
                    return Err("invalid bit depth for color type");
                }
            },
            _ => {
                return Err("invalid color type");
            }
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
pub struct PLTE {
    red: u8,
    green: u8,
    blue: u8,
    filter_method: u8,
    interlace_method: u8,
}

enum Unit {
    Unknown = 0,
    Meter = 1,
}

pub struct pHYs {
    x: u8,
    y: u8,
    unit: Unit,
}

pub struct AncillaryChunks {
    phys: Option<pHYs>
}
