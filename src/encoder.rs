use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::chunks::Chunk;
use crate::common::{HEADER, IEND};
use crate::errors::PngDecodingError;
use crate::png::Png;

impl Png {
    pub fn save<S: AsRef<Path>>(png: Png, file_path: S) -> Result<(), PngDecodingError> {
        let buffer = &mut BufWriter::new(File::create(file_path)?);
        Self::write(png, buffer)?;
        Ok(())
    }

    pub fn write<T: Write>(png: Png, buffer: &mut BufWriter<T>) -> Result<(), PngDecodingError> {
        buffer.write_all(&HEADER)?;
        buffer.write_all(&[0u8, 0, 0, 13])?;
        buffer.write_all(&png.ihdr.into_bytes())?;
        buffer.write_all(&IEND)?;
        Ok(())
    }
}
