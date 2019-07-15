#[derive(Debug, PartialEq)]
pub enum BitDepth {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
}

impl BitDepth {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::One,
            2 => Self::Two,
            4 => Self::Four,
            8 => Self::Eight,
            16 => Self::Sixteen,
            _ => panic!("unrecognized bit depth")
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
            Self::Eight => 8,
            Self::Sixteen => 16,
        }
    }
}

impl std::default::Default for BitDepth {
    fn default() -> Self {
        Self::Eight
    }
}

#[derive(Debug)]
pub enum CompressionType {
    Deflate = 0,
}

impl CompressionType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Deflate,
            _ => panic!("unrecognized compression type")
        }
    }
}

impl std::default::Default for CompressionType {
    fn default() -> Self {
        Self::Deflate
    }
}

#[derive(Debug)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2, // Truecolor
    Indexed = 3,
    GrayscaleAlpha = 4,
    RGBA = 6, // TruecolorAlpha
}

impl std::default::Default for ColorType {
    fn default() -> Self {
        ColorType::RGBA
    }
}

impl ColorType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Grayscale,
            2 => Self::RGB,
            3 => Self::Indexed,
            4 => Self::GrayscaleAlpha,
            6 => Self::RGBA,
            _ => panic!("unrecognized color type")
        }
    }
}

#[derive(Debug)]
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
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Meters,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

#[derive(Debug)]
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
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Adam7,
            _ => panic!("Unknown value: {}", value),
        }
    }
}