use std::{
    fmt,
    io::{BufRead, Read},
    ops::Index,
};

use crate::{
    common::ColorType,
    errors::{ChunkError, MetadataError, PngDecodingError},
};

/// The IHDR chunk contains important metadata for reading the image
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_type: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

impl IHDR {
    pub fn new(
        width: u32,
        height: u32,
        bit_depth: u8,
        color_type: ColorType,
        compression_type: u8,
        filter_method: u8,
        interlace_method: u8,
    ) -> Result<Self, MetadataError> {
        if !(0 < width && width < 2u32.pow(31)) {
            // between 0 and 2**31
            return Err(MetadataError::InvalidWidth {
                width: width as usize,
            });
        }

        if !(0 < height && height < 2u32.pow(31)) {
            // between 0 and 2**31
            return Err(MetadataError::InvalidHeight {
                height: height as usize,
            });
        }

        static VALID_COLOR_TYPES: [&[u8]; 7] = [
            // grayscale (0)
            &[1, 2, 4, 8, 16],
            &[],
            // rgb (2)
            &[8, 16],
            // indexed (3)
            &[1, 2, 4, 8],
            // grayscale alpha (4)
            &[8, 16],
            &[],
            // rgba (6)
            &[8, 16],
        ];

        if !VALID_COLOR_TYPES[color_type as usize].contains(&bit_depth) {
            return Err(MetadataError::InvalidBitDepthForColorType {
                bit_depth,
                color_type,
            });
        }

        Ok(Self {
            width,
            height,
            bit_depth,
            color_type,
            compression_type,
            filter_method,
            interlace_method,
        })
    }
}

impl<'a> NamedChunk<'a> for IHDR {
    const NAME: [u8; 4] = *b"IHDR";
}

impl<'a> Chunk<'a> for IHDR {
    fn parse<T: Read + BufRead>(length: u32, buf: &mut T) -> Result<Self, PngDecodingError> {
        let (mut width_buffer, mut height_buffer) = ([0u8; 4], [0u8; 4]);
        let (
            mut bit_depth_buffer,
            mut color_type_buffer,
            mut compression_type_buffer,
            mut filter_method_buffer,
            mut interlace_method_buffer,
        ) = ([0u8; 1], [0u8; 1], [0u8; 1], [0u8; 1], [0u8; 1]);

        if length != 13 {
            return Err(PngDecodingError::InvalidIHDRLength(length));
        }

        buf.read_exact(&mut width_buffer)?;
        let width = u32::from_be_bytes(width_buffer);

        buf.read_exact(&mut height_buffer)?;
        let height = u32::from_be_bytes(height_buffer);

        buf.read_exact(&mut bit_depth_buffer)?;
        let bit_depth = u8::from_be_bytes(bit_depth_buffer);

        buf.read_exact(&mut color_type_buffer)?;
        let color_type = ColorType::from_u8(u8::from_be_bytes(color_type_buffer))?;

        buf.read_exact(&mut compression_type_buffer)?;
        let compression_type = u8::from_be_bytes(compression_type_buffer);

        buf.read_exact(&mut filter_method_buffer)?;
        let filter_method = u8::from_be_bytes(filter_method_buffer);

        buf.read_exact(&mut interlace_method_buffer)?;
        let interlace_method = u8::from_be_bytes(interlace_method_buffer);

        Ok(IHDR::new(
            width,
            height,
            bit_depth,
            color_type,
            compression_type,
            filter_method,
            interlace_method,
        )?)
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.width.to_be_bytes());
        buffer.extend_from_slice(&self.height.to_be_bytes());
        buffer.push(self.bit_depth);
        buffer.push(self.color_type as u8);
        buffer.push(self.compression_type);
        buffer.push(self.filter_method);
        buffer.push(self.interlace_method);
    }
}

const fn is_upper(b: u8) -> bool {
    (b & 0b0010_0000) == 0
}

pub trait Chunk<'a> {
    fn parse<T: Read + BufRead>(length: u32, buf: &mut T) -> Result<Self, PngDecodingError>
    where
        Self: Sized;
    fn serialize(&self, buffer: &mut Vec<u8>);

    fn size_hint(&self) -> usize
    where
        Self: Sized,
    {
        std::mem::size_of::<Self>()
    }
}

pub trait NamedChunk<'a>: Chunk<'a> {
    const NAME: [u8; 4];

    const IS_CRITICAL: bool = is_upper(Self::NAME[0]);
    const IS_PUBLIC: bool = is_upper(Self::NAME[1]);
    const IS_RESERVED_FIELD: bool = is_upper(Self::NAME[2]);
    const IS_SAFE_TO_COPY: bool = is_upper(Self::NAME[3]);
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct UnrecognizedChunk {
    pub length: u32,
    pub chunk_type: [u8; 4],
    pub bytes: Vec<u8>,
    pub is_critical: bool,
    pub is_public: bool,
    pub is_safe_to_copy: bool,
}

impl fmt::Debug for UnrecognizedChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnrecognizedChunk")
            .field("length", &self.length)
            .field("chunk_type", &self.chunk_type)
            .finish()
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
        vec![self.red, self.green, self.blue]
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
    pub entries: Vec<PaletteEntry>,
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

impl<'a> NamedChunk<'a> for PLTE {
    const NAME: [u8; 4] = *b"PLTE";
}

impl<'a> Chunk<'a> for PLTE {
    fn parse<T: Read + BufRead>(length: u32, buf: &mut T) -> Result<Self, PngDecodingError> {
        if length % 3 != 0 {
            return Err(ChunkError::InvalidPLTELength.into());
        }
        let mut entries_buffer: Vec<u8> = vec![0; length as usize];
        buf.read_exact(&mut entries_buffer)?;
        let entries_: Vec<&[u8]> = entries_buffer.chunks_exact(3).collect();
        let entries: Vec<PaletteEntry> = entries_.iter().map(|x| PaletteEntry::from(*x)).collect();

        Ok(PLTE { entries })
    }

    fn serialize(&self, _buffer: &mut Vec<u8>) {
        todo!()
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

impl<'a> NamedChunk<'a> for pHYs {
    const NAME: [u8; 4] = *b"pHYs";
}

impl<'a> Chunk<'a> for pHYs {
    fn parse<T: Read + BufRead>(_length: u32, buf: &mut T) -> Result<Self, PngDecodingError> {
        let mut pixels_per_x_buffer = [0u8; 4];
        let mut pixels_per_y_buffer = [0u8; 4];
        let mut unit_buffer = [0u8];

        buf.read_exact(&mut pixels_per_x_buffer)?;
        let pixels_per_unit_x = u32::from_be_bytes(pixels_per_x_buffer);

        buf.read_exact(&mut pixels_per_y_buffer)?;
        let pixels_per_unit_y = u32::from_be_bytes(pixels_per_y_buffer);

        buf.read_exact(&mut unit_buffer)?;
        let unit = u8::from_be_bytes(unit_buffer);

        Ok(pHYs {
            pixels_per_unit_x,
            pixels_per_unit_y,
            unit: Unit::from_u8(unit)?,
        })
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.pixels_per_unit_x.to_be_bytes());
        buffer.extend_from_slice(&self.pixels_per_unit_y.to_be_bytes());
        buffer.push(self.unit as u8);
    }
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
            _ => Err(MetadataError::UnrecognizedUnit { unit }),
        }
    }
}

/// The tEXt chunk contains uncompressed, Latin-1 encoded textual information
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct tEXt {
    // todo: vec<u8> and include nullbyte, or change serialization
    pub keyword: String,
    pub text: String,
}

impl<'a> NamedChunk<'a> for tEXt {
    const NAME: [u8; 4] = *b"tEXt";
}

impl<'a> Chunk<'a> for tEXt {
    fn parse<T: Read + BufRead>(length: u32, buf: &mut T) -> Result<Self, PngDecodingError> {
        let mut keyword_buffer: Vec<u8> = Vec::new();
        let keyword_len = buf.read_until(b'\0', &mut keyword_buffer)?;

        let remaining_length = length - (keyword_len as u32);

        let mut text_buffer: Vec<u8> = vec![0; remaining_length as usize];
        buf.read_exact(&mut text_buffer)?;

        // the null byte is included in `read_until()`
        keyword_buffer.pop();

        // let keyword = if let Ok(k) = String::from_utf8(keyword_buffer) { k } else { continue };
        let keyword = String::from_utf8(keyword_buffer)?;
        let text = String::from_utf8(text_buffer)?;

        Ok(tEXt { keyword, text })
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(self.keyword.as_bytes());
        buffer.extend_from_slice(self.text.as_bytes());
    }

    fn size_hint(&self) -> usize
    where
        Self: Sized,
    {
        self.keyword.len() + self.text.len()
    }
}

/// The iTXt chunk contains optionally compressed, UTF-8 encoded text
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct iTXt {
    pub keyword: String,
    pub compressed: bool, // compression flag: 0=false; 1=true
    pub compression_method: Option<u8>,
    pub language_tag: String,
    pub translated_keyword: String,
    pub text: String,
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct gAMA {
    pub gamma: u32,
}

impl<'a> NamedChunk<'a> for gAMA {
    const NAME: [u8; 4] = *b"gAMA";
}

impl<'a> Chunk<'a> for gAMA {
    fn parse<T: Read + BufRead>(length: u32, buf: &mut T) -> Result<Self, PngDecodingError> {
        if length != 4 {
            return Err(ChunkError::InvalidgAMALength.into());
        }
        let mut gamma_buffer = [0u8; 4];
        buf.read_exact(&mut gamma_buffer)?;
        let gamma = u32::from_be_bytes(gamma_buffer);
        Ok(gAMA { gamma })
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.gamma.to_be_bytes());
    }
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

impl<'a> NamedChunk<'a> for cHRM {
    const NAME: [u8; 4] = *b"cHRM";
}

impl<'a> Chunk<'a> for cHRM {
    fn parse<T: Read + BufRead>(_length: u32, buf: &mut T) -> Result<Self, PngDecodingError>
    where
        Self: Sized,
    {
        let (
            mut white_point_x_buffer,
            mut white_point_y_buffer,
            mut red_x_buffer,
            mut red_y_buffer,
            mut green_x_buffer,
            mut green_y_buffer,
            mut blue_x_buffer,
            mut blue_y_buffer,
        ) = (
            [0u8; 4], [0u8; 4], [0u8; 4], [0u8; 4], [0u8; 4], [0u8; 4], [0u8; 4], [0u8; 4],
        );

        buf.read_exact(&mut white_point_x_buffer)?;
        let white_point_x = u32::from_be_bytes(white_point_x_buffer);

        buf.read_exact(&mut white_point_y_buffer)?;
        let white_point_y = u32::from_be_bytes(white_point_y_buffer);

        buf.read_exact(&mut red_x_buffer)?;
        let red_x = u32::from_be_bytes(red_x_buffer);

        buf.read_exact(&mut red_y_buffer)?;
        let red_y = u32::from_be_bytes(red_y_buffer);

        buf.read_exact(&mut green_x_buffer)?;
        let green_x = u32::from_be_bytes(green_x_buffer);

        buf.read_exact(&mut green_y_buffer)?;
        let green_y = u32::from_be_bytes(green_y_buffer);

        buf.read_exact(&mut blue_x_buffer)?;
        let blue_x = u32::from_be_bytes(blue_x_buffer);

        buf.read_exact(&mut blue_y_buffer)?;
        let blue_y = u32::from_be_bytes(blue_y_buffer);

        Ok(cHRM {
            white_point_x,
            white_point_y,
            red_x,
            red_y,
            green_x,
            green_y,
            blue_x,
            blue_y,
        })
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.white_point_x.to_be_bytes());
        buffer.extend_from_slice(&self.white_point_y.to_be_bytes());
        buffer.extend_from_slice(&self.red_x.to_be_bytes());
        buffer.extend_from_slice(&self.red_y.to_be_bytes());
        buffer.extend_from_slice(&self.green_x.to_be_bytes());
        buffer.extend_from_slice(&self.green_y.to_be_bytes());
        buffer.extend_from_slice(&self.blue_x.to_be_bytes());
        buffer.extend_from_slice(&self.blue_y.to_be_bytes());
    }
}

/// Contains information for image's [ICC profile](https://en.wikipedia.org/wiki/ICC_profile)
#[derive(Default, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct iCCP {
    pub profile_name: Vec<u8>,
    pub compression_method: u8,
    pub compressed_profile: Vec<u8>,
}

impl<'a> Chunk<'a> for iCCP {
    fn parse<T: Read + BufRead>(length: u32, buf: &mut T) -> Result<Self, PngDecodingError>
    where
        Self: Sized,
    {
        let mut profile_name: Vec<u8> = Vec::new();
        let mut compression_method_buffer = [0];

        let profile_name_len = buf.read_until(b'\0', &mut profile_name)?;
        buf.read_exact(&mut compression_method_buffer)?;

        let compression_method = u8::from_be_bytes(compression_method_buffer);

        let remaining_length = length - (profile_name_len as u32) - 1;

        let mut compressed_profile: Vec<u8> = vec![0; remaining_length as usize];
        buf.read_exact(&mut compressed_profile)?;

        Ok(iCCP {
            profile_name,
            compression_method,
            compressed_profile,
        })
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.profile_name);
        buffer.push(self.compression_method);
        buffer.extend_from_slice(&self.compressed_profile);
    }

    fn size_hint(&self) -> usize
    where
        Self: Sized,
    {
        self.profile_name.len() + 1 + self.compressed_profile.len()
    }
}

impl<'a> NamedChunk<'a> for iCCP {
    const NAME: [u8; 4] = *b"iCCP";
}

impl fmt::Debug for iCCP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "iCCP {{ {} }}",
            String::from_utf8_lossy(&self.profile_name)
        )
    }
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ICCProfile {
    icc_profile: Vec<u8>,
}

/// Contains the number of significant bits.
/// It is useful for scaling color precision
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum sBIT {
    Grayscale {
        grayscale: u8,
    },
    RGB {
        red: u8,
        green: u8,
        blue: u8,
    },
    Indexed {
        red: u8,
        green: u8,
        blue: u8,
    },
    GrayscaleAlpha {
        grayscale: u8,
        alpha: u8,
    },
    RGBA {
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    },
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
    Grayscale { grayscale: u16 },
    RGB { red: u16, green: u16, blue: u16 },
    Indexed { entries: Vec<u8> },
}

impl fmt::Debug for tRNS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            tRNS::Grayscale { grayscale } => f
                .debug_struct("tRNS::Grayscale")
                .field("grayscale", grayscale)
                .finish(),
            tRNS::RGB { red, green, blue } => f
                .debug_struct("tRNS::RGB")
                .field("red", red)
                .field("green", green)
                .field("blue", blue)
                .finish(),
            tRNS::Indexed { entries } => write!(f, "tRNS {{ {} entries }}", entries.len()),
        }
    }
}

/// Contains default background color
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum bKGD {
    // TODO: rename field to gray
    Grayscale {
        grayscale: u16,
    },
    RGB {
        red: u16,
        green: u16,
        blue: u16,
    },
    Palette {
        palette_index: u8,
        rgb: PaletteEntry,
    },
}

impl bKGD {
    pub fn rgb(self) -> [u16; 3] {
        match self {
            bKGD::Grayscale { grayscale } => [grayscale, grayscale, grayscale],
            bKGD::RGB { red, green, blue } => [red, green, blue],
            bKGD::Palette { rgb, .. } => rgb.to_array(),
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
        if $self.$chunk.is_none() {
            String::from("")
        } else {
            format!("\n\t{:?}", $self.$chunk)
        }
    };
}

macro_rules! show_optional_chunk_mult {
    ($self:ident, $chunk:ident) => {
        if $self.$chunk.is_empty() {
            String::from("")
        } else {
            format!("\n\t{:?}", $self.$chunk)
        }
    };
}

impl fmt::Display for AncillaryChunks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AncillaryChunks {{ {}{}{}{}{}{}{}{}{}\n    }}",
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
