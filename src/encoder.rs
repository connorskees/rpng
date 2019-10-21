#![allow(dead_code, unused_must_use, unused_variables)]

use std::io::{Write, BufWriter};
use std::path::Path;
use std::fs::File;

use flate2::write::ZlibEncoder;
use flate2::Compression;
use crc32fast::Hasher;

use crate::errors::PNGDecodingError;
use crate::png::PNG;
use crate::common::{HEADER, IEND};
use crate::chunks::Chunk;
use crate::utils::u32_to_be_bytes;

pub fn save<S: AsRef<Path>>(png: PNG, file_path: S) -> Result<(), PNGDecodingError> {
    let buffer = &mut BufWriter::new(File::create(file_path)?);
    write(png, buffer);
    Ok(())
}

pub fn write<T: Write>(png: PNG, buffer: &mut BufWriter<T>) -> Result<(), PNGDecodingError> {
    buffer.write_all(&HEADER)?;
    buffer.write_all(&[0u8, 0, 0, 13])?;
    buffer.write_all(&png.ihdr.as_bytes())?;
    // let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    // encoder.write_all(&png.idat);

    for chunk in png.idat.chunks(std::usize::MAX) {//encoder.finish()?.chunks(std::usize::MAX) {
        // buffer.write_all(png.idat.len())?;
        buffer.write_all(b"IDAT")?;
        buffer.write_all(chunk)?;

        let mut hasher = Hasher::new();
        hasher.update(b"IDAT");
        hasher.update(&chunk);
        
        buffer.write_all(&u32_to_be_bytes(hasher.finalize()));        
    }
    buffer.write_all(&IEND)?;
    Ok(())
}