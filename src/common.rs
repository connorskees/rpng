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
            0 => ColorType::Grayscale,
            2 => ColorType::RGB,
            3 => ColorType::Indexed,
            4 => ColorType::GrayscaleAlpha,
            6 => ColorType::RGBA,
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
pub enum Interlace {
    None = 0,
    Adam7 = 1,
}

impl std::default::Default for Interlace {
    fn default() -> Self {
        Self::None
    }
}

impl Interlace {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Adam7,
            _ => panic!("Unknown value: {}", value),
        }
    }
}