#![allow(dead_code, unused_must_use, unused_variables)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::chunks::Chunk;
use crate::common::{HEADER, IEND};
use crate::errors::PNGDecodingError;
use crate::png::PNG;

pub fn save<S: AsRef<Path>>(png: PNG, file_path: S) -> Result<(), PNGDecodingError> {
    let buffer = &mut BufWriter::new(File::create(file_path)?);
    write(png, buffer);
    Ok(())
}

pub fn write<T: Write>(png: PNG, buffer: &mut BufWriter<T>) -> Result<(), PNGDecodingError> {
    buffer.write_all(&HEADER)?;
    buffer.write_all(&[0u8, 0, 0, 13])?;
    buffer.write_all(&png.ihdr.into_bytes())?;
    buffer.write_all(&IEND)?;
    Ok(())
}
