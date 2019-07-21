use std::fmt;
use crate::common::{BitDepth, ColorType, CompressionType, Unit};
use crate::filter::{FilterMethod};
use crate::interlacing::{Interlacing};
use crate::errors::MetadataError;

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
        ) -> Result<Self, MetadataError> {

        if !(0 < width && width < 2u32.pow(31)) {
            // between 0 and 2^31
            return Err(MetadataError::InvalidWidth{ width });
        }
        
        if !(0 < height && height < 2u32.pow(31)) {
            // between 0 and 2^31
            return Err(MetadataError::InvalidHeight{ height });
        }

        match color_type {
            ColorType::Grayscale => {
                if ![BitDepth::One, BitDepth::Two, BitDepth::Four, BitDepth::Eight, BitDepth::Sixteen].contains(&bit_depth) {
                    return Err(MetadataError::InvalidBitDepthForColorType{ bit_depth, color_type });
                }
            },
            ColorType::RGB => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&bit_depth) {
                    return Err(MetadataError::InvalidBitDepthForColorType{ bit_depth, color_type }); 
                }
            },
            ColorType::Indexed => {
                if ![BitDepth::One, BitDepth::Two, BitDepth::Four, BitDepth::Eight].contains(&bit_depth) {
                    return Err(MetadataError::InvalidBitDepthForColorType{ bit_depth, color_type });
                }
            },
            ColorType::GrayscaleAlpha => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&bit_depth) {
                    return Err(MetadataError::InvalidBitDepthForColorType{ bit_depth, color_type });
                }
            },
            ColorType::RGBA => {
                if ![BitDepth::Eight, BitDepth::Sixteen].contains(&bit_depth) {
                    return Err(MetadataError::InvalidBitDepthForColorType{ bit_depth, color_type });
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
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl From<&[u8]> for PaletteEntry {
    fn from(val: &[u8]) -> Self {
        PaletteEntry {
            red: u16::from(val[0]),
            green: u16::from(val[1]),
            blue: u16::from(val[2]),
        }
    }
}

impl From<[u8; 3]> for PaletteEntry {
    fn from(val: [u8; 3]) -> Self {
        PaletteEntry {
            red: u16::from(val[0]),
            green: u16::from(val[1]),
            blue: u16::from(val[2]),
        }
    }
}

impl From<[u16; 3]> for PaletteEntry {
    fn from(val: [u16; 3]) -> Self {
        PaletteEntry {
            red: val[0],
            green: val[1],
            blue: val[2],
        }
    }
}


impl PaletteEntry {
    /// Return the RGB value as a vector
    pub fn to_vec(&self) -> Vec<u16> {
        vec!(self.red, self.green, self.blue)
    }

    /// Return the RGB value as an array [u8; 3]
    pub fn to_array(&self) -> [u16; 3] {
        [self.red, self.green, self.blue]
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

impl Index<u8> for PLTE {
    type Output = PaletteEntry;

    fn index(&self, index: u8) -> &Self::Output {
        &self.entries[usize::from(index)]
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

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct tEXt {
    pub keyword: String,
    pub text: String,
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

/// Contains the number of significant bits.
/// It is useful for scaling color precision
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum sBIT {
    Grayscale{grayscale: u8},
    RGB{red: u8, green: u8, blue: u8},
    Indexed{red: u8, green: u8, blue: u8},
    GrayscaleAlpha{grayscale: u8, alpha: u8},
    RGBA{red: u8, green: u8, blue: u8, alpha: u8},
}

/// Contains information about the ICC specified rendering intent
/// http://www.libpng.org/pub/png/spec/1.2/PNG-Chunks.html#C.sRGB
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum sRGB {
    /// Perceptual intent is for images preferring good adaptation to the output
    /// device gamut at the expense of colorimetric accuracy, like photographs
    Perceptual = 0,
    /// Relative colorimetric intent is for images requiring color appearance 
    /// matching (relative to the output device white point), like logos
    RelativeColorimetric = 1,
    /// Saturation intent is for images preferring preservation of saturation
    /// at the expense of hue and lightness, like charts and graphs
    Saturation = 2,
    /// Absolute colorimetric intent is for images requiring preservation of
    /// absolute colorimetry, like proofs (previews of images destined for a
    /// different output device)
    AbsoluteColorimetric = 3,
}

impl sRGB {
    pub fn from_u8(val: u8) -> Result<sRGB, ChunkError> {
        match val {
            0 => Ok(sRGB::Perceptual),
            1 => Ok(sRGB::RelativeColorimetric),
            2 => Ok(sRGB::Saturation),
            3 => Ok(sRGB::AbsoluteColorimetric),
            _ => Err(ChunkError::UnrecognizedsRGBValue(val)),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum bKGD {
    Grayscale{grayscale: u16},
    RGB{red: u16, green: u16, blue: u16},
    Palette{palette_index: u8},
}

/// Ancillary chunks are those that are not necessary to render the image
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct AncillaryChunks {
    pub phys: Option<pHYs>,
    pub itxt: Vec<Option<iTXt>>,
    pub gama: Option<gAMA>,
    pub chrm: Option<cHRM>,
    pub iccp: Option<iCCP>,
    pub tEXt: Vec<Option<tEXt>>,
    pub bKGD: Option<bKGD>,
    pub sBIT: Option<sBIT>,
    pub sRGB: Option<sRGB>,
}

impl AncillaryChunks {
    pub fn new() -> AncillaryChunks {
        AncillaryChunks {
            phys: None,
            itxt: Vec::new(),
            gama: None,
            chrm: None,
            iccp: None,
            tEXt: Vec::new(),
            bKGD: None,
            sBIT: None,
            sRGB: None,
        }
    }
}

impl std::default::Default for AncillaryChunks {
    fn default() -> Self {
        AncillaryChunks::new()
    }
}
