use std::fmt;
use std::ops::Index;
use std::io::{Read, BufRead};

use crc32fast::Hasher;

use crate::common::{BitDepth, ColorType, CompressionType};
use crate::filter::{FilterMethod};
use crate::interlacing::{Interlacing};
use crate::errors::{ChunkError, MetadataError, PNGDecodingError};

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
            // between 0 and 2**31
            return Err(MetadataError::InvalidWidth{ width: width as usize });
        }
        
        if !(0 < height && height < 2u32.pow(31)) {
            // between 0 and 2**31
            return Err(MetadataError::InvalidHeight{ height: height as usize });
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

impl<'a> Chunk<'a> for IHDR {
    const IS_CRITICAL: bool = true;
    const IS_PUBLIC: bool = true;
    const IS_SAFE_TO_COPY: bool = true;
    const NAME: &'a str = "IHDR";

    fn parse<T: Read + BufRead>(length: u32, buf: &mut T) -> Result<Self, PNGDecodingError> {
        let (
            mut width_buffer,
            mut height_buffer,
        ) = ([0; 4], [0; 4]);
        let (
            mut bit_depth_buffer,
            mut color_type_buffer,
            mut compression_type_buffer,
            mut filter_method_buffer,
            mut interlace_method_buffer
        ) = ([0; 1], [0; 1], [0; 1], [0; 1], [0; 1]);

        if length != 13 {
            return Err(PNGDecodingError::InvalidIHDRLength(length));
        }

        buf.read_exact(&mut width_buffer)?;
        let width = u32::from_be_bytes(width_buffer);
        
        buf.read_exact(&mut height_buffer)?;
        let height = u32::from_be_bytes(height_buffer);
        
        buf.read_exact(&mut bit_depth_buffer)?;
        let bit_depth = BitDepth::from_u8(u8::from_be_bytes(bit_depth_buffer))?;
        
        buf.read_exact(&mut color_type_buffer)?;
        let color_type = ColorType::from_u8(u8::from_be_bytes(color_type_buffer))?;
        
        buf.read_exact(&mut compression_type_buffer)?;
        let compression_type = CompressionType::from_u8(u8::from_be_bytes(compression_type_buffer))?;
        
        buf.read_exact(&mut filter_method_buffer)?;
        let filter_method = FilterMethod::from_u8(u8::from_be_bytes(filter_method_buffer))?;
        
        buf.read_exact(&mut interlace_method_buffer)?;
        let interlace_method = Interlacing::from_u8(u8::from_be_bytes(interlace_method_buffer))?;

        Ok(IHDR::new(width, height, bit_depth, color_type, compression_type, filter_method, interlace_method)?)
    }

    fn as_bytes(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::with_capacity(4+13+4);
        
        buffer.extend(b"IHDR");
        buffer.extend(&u32_to_be_bytes(self.width));
        buffer.extend(&u32_to_be_bytes(self.height));
        buffer.push(self.bit_depth.as_u8());
        buffer.push(self.color_type.as_u8());
        buffer.push(self.compression_type.as_u8());
        buffer.push(self.filter_method.as_u8());
        buffer.push(self.interlace_method.as_u8());
        buffer.push(self.interlace_method.as_u8());

        let mut hasher = Hasher::new();
        hasher.update(&buffer);
        buffer.extend(&u32_to_be_bytes(hasher.finalize()));
        assert_eq!(buffer.len(), 21);

        buffer
    }
}

trait Chunk<'a> {
    const IS_CRITICAL: bool;
    const IS_PUBLIC: bool;
    const IS_RESERVED_FIELD: bool = false;
    const IS_SAFE_TO_COPY: bool;
    const NAME: &'a str;
    fn parse<T: Read + BufRead>(length: u32, buf: &mut T) -> Result<Self, PNGDecodingError> where Self: std::marker::Sized;
    fn as_bytes(self) -> Vec<u8>;
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
    pub fn to_vec(self) -> Vec<u16> {
        vec!(self.red, self.green, self.blue)
    }

    /// Return the RGB value as an array [u8; 3]
    pub const fn to_array(self) -> [u16; 3] {
        [self.red, self.green, self.blue]
    }   
}

/// The PLTE chunk contains a list of palette entries.
/// Entries are 0 indexed
#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub struct PLTE {
    pub entries: Vec<PaletteEntry>
}

impl fmt::Display for PLTE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PLTE {{ {} entries }}", self.entries.len())
    }
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

impl Index<u16> for PLTE {
    type Output = PaletteEntry;

    fn index(&self, index: u16) -> &Self::Output {
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum Unit {
    Unknown = 0,
    Meters = 1,
}

impl std::default::Default for Unit {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Unit {
    pub fn from_u8(unit: u8) -> Result<Self, MetadataError> {
        match unit {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Meters),
            _ => Err(MetadataError::UnrecognizedUnit{ unit }),
        }
    }
}

/// The tEXt chunk contains uncompressed, Latin-1 encoded textual information
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct tEXt {
    pub keyword: String,
    pub text: String,
}

/// The iTXt chunk contains optionally compressed, UTF-8 encoded text
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

/// Contains information for image's [ICC profile](https://en.wikipedia.org/wiki/ICC_profile) 
#[derive(Default, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct iCCP {
    pub profile_name: String,
    pub compression_method: CompressionType,
    pub compressed_profile: Vec<u8>,
}

impl fmt::Debug for iCCP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "iCCP {{ {} }}", self.profile_name)
    }
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ICCProfile {
    icc_profile: Vec<u8>
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
/// [spec](http://www.libpng.org/pub/png/spec/1.2/PNG-Chunks.html#C.sRGB)
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

/// Contains transparency information
#[derive(Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum tRNS {
    Grayscale{ grayscale: u16 },
    RGB{ red: u16, green: u16, blue: u16 },
    Indexed{ entries: Vec<u8> },
}

impl fmt::Debug for tRNS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use tRNS::*;
        match self {
            Grayscale{ .. } => write!(f, "{:?}", self),
            RGB{ .. } => write!(f, "{:?}", self),
            Indexed{ entries } => write!(f, "tRNS {{ {} entries }}", entries.len())
        }
    }
}

/// Contains default background color
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum bKGD {
    // TODO: rename field to gray
    Grayscale{ grayscale: u16 },
    RGB{ red: u16, green: u16, blue: u16 },
    Palette{ palette_index: u8, rgb: PaletteEntry },
}

impl bKGD {
    pub fn rgb(self) -> [u16; 3] {
        match self {
            bKGD::Grayscale{ grayscale } => {
                [grayscale, grayscale, grayscale]
            },
            bKGD::RGB{ red, green, blue } => {
                [red, green, blue]
            },
            bKGD::Palette{ rgb, .. } => {
                rgb.to_array()
            }
        }
    }
}

/// Ancillary chunks are those that are not necessary to render the image
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct AncillaryChunks {
    pub pHYs: Option<pHYs>,
    pub itxt: Vec<iTXt>,
    pub gama: Option<gAMA>,
    pub chrm: Option<cHRM>,
    pub iCCP: Option<iCCP>,
    pub tEXt: Vec<tEXt>,
    pub bKGD: Option<bKGD>,
    pub sBIT: Option<sBIT>,
    pub sRGB: Option<sRGB>,
    pub tRNS: Option<tRNS>,
}

impl AncillaryChunks {
    pub fn new() -> AncillaryChunks {
        AncillaryChunks {
            pHYs: None,
            itxt: Vec::new(),
            gama: None,
            chrm: None,
            iCCP: None,
            tEXt: Vec::new(),
            bKGD: None,
            sBIT: None,
            sRGB: None,
            tRNS: None,
        }
    }
}

macro_rules! show_optional_chunk {
    ($self:ident, $chunk:ident) => {
        if $self.$chunk.is_none() { String::from("") } else { format!("\n\t{:?}", $self.$chunk) }
    };
}

macro_rules! show_optional_chunk_mult {
    ($self:ident, $chunk:ident) => {
        if $self.$chunk.is_empty() { String::from("") } else { format!("\n\t{:?}", $self.$chunk) }
    };
}

impl fmt::Display for AncillaryChunks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "AncillaryChunks {{ {}{}{}{}{}{}{}{}{}\n    }}",
            show_optional_chunk!(self, pHYs),
            show_optional_chunk_mult!(self, itxt),
            show_optional_chunk!(self, gama),
            show_optional_chunk!(self, chrm),
            show_optional_chunk!(self, iCCP),
            show_optional_chunk_mult!(self, tEXt),
            show_optional_chunk!(self, bKGD),
            show_optional_chunk!(self, sBIT),
            show_optional_chunk!(self, sRGB),
        )
    }
}

impl std::default::Default for AncillaryChunks {
    fn default() -> Self {
        AncillaryChunks::new()
    }
}

fn u32_to_be_bytes(num: u32) -> [u8; 4] {
    #[allow(unsafe_code)]
    unsafe {
        std::mem::transmute::<u32, [u8; 4]>(num.to_be())
    }
}