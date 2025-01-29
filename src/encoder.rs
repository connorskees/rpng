use std::borrow::Cow;
use std::fs::File;
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::Path;

use crc32fast::Hasher;
use flate2::bufread::ZlibEncoder;
use flate2::Compression;

use crate::chunks::{Chunk, NamedChunk};
use crate::common::{HEADER, IEND};
use crate::errors::PngDecodingError;
use crate::png::Png;

impl Png {
    pub fn save<S: AsRef<Path>>(&self, file_path: S) -> Result<(), PngDecodingError> {
        let buffer = &mut BufWriter::new(File::create(file_path)?);
        self.write(buffer)?;
        Ok(())
    }

    pub fn write<T: Write>(&self, buffer: &mut BufWriter<T>) -> Result<(), PngDecodingError> {
        buffer.write_all(&HEADER)?;
        self.write_chunk(&self.ihdr, buffer)?;

        if let Some(chrm) = &self.ancillary_chunks.chrm {
            self.write_chunk(chrm, buffer)?;
        }
        if let Some(iccp) = &self.ancillary_chunks.iCCP {
            self.write_chunk(iccp, buffer)?;
        }

        self.write_data(buffer)?;

        buffer.write_all(&IEND)?;
        Ok(())
    }

    fn write_chunk<'a, T: Write, C: NamedChunk<'a>>(
        &self,
        chunk: &C,
        buffer: &mut BufWriter<T>,
    ) -> Result<(), PngDecodingError> {
        let mut serialized = Vec::with_capacity(4 + chunk.size_hint());
        serialized.extend_from_slice(&C::NAME);
        chunk.serialize(&mut serialized);

        let len = serialized.len() as u32 - 4;

        buffer.write_all(&len.to_be_bytes())?;
        buffer.write_all(&serialized)?;

        let mut hasher = Hasher::new();
        hasher.update(&serialized);
        buffer.write_all(&hasher.finalize().to_be_bytes())?;

        Ok(())
    }

    fn write_data<T: Write>(&self, buffer: &mut BufWriter<T>) -> Result<(), PngDecodingError> {
        let chunk = DataChunk {
            width: self.width(),
            height: self.height(),
            bpp: self.bpp(),
            raw_buffer: self
                .decoded_buffer
                .as_deref()
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(self.decode().buffer)),
        };

        self.write_chunk(&chunk, buffer)?;

        Ok(())
    }
}

struct DataChunk<'a> {
    raw_buffer: Cow<'a, [u8]>,
    width: u32,
    height: u32,
    bpp: usize,
}

impl<'a> NamedChunk<'a> for DataChunk<'_> {
    const NAME: [u8; 4] = *b"IDAT";
}

impl<'a> Chunk<'a> for DataChunk<'_> {
    fn parse<T: Read + std::io::prelude::BufRead>(
        _length: u32,
        _buf: &mut T,
    ) -> Result<Self, PngDecodingError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn serialize(&self, out_buffer: &mut Vec<u8>) {
        let filtered = self.filter_image();

        let mut compressor = ZlibEncoder::new(Cursor::new(filtered), Compression::fast());
        compressor.read_to_end(out_buffer).unwrap();
    }

    fn size_hint(&self) -> usize
    where
        Self: Sized,
    {
        self.raw_buffer.len() + self.height as usize
    }
}

impl DataChunk<'_> {
    fn filter_image(&self) -> Vec<u8> {
        let mut out =
            vec![0; self.bpp * self.width as usize * self.height as usize + self.height as usize];

        let mut offset = 0;
        let up = vec![0; self.bpp * self.width as usize];
        let mut up: &[u8] = &up;

        for row in self.raw_buffer.chunks_exact(self.width as usize * self.bpp) {
            self.filter_row(row, up, &mut out[offset..]);
            offset += row.len() + 1;
            up = row;
        }

        out
    }

    fn filter_row(&self, row: &[u8], up: &[u8], out: &mut [u8]) {
        let (filter_cell, out) = out.split_at_mut(1);

        let sum = |row: &[u8]| {
            row.iter()
                .map(|&b| i8::from_be_bytes([b]).wrapping_abs() as i32)
                .sum::<i32>()
        };

        self.filter_none(row, out);
        let none_sum = sum(out);

        self.filter_sub(row, out);
        let sub_sum = sum(out);

        self.filter_up(row, up, out);
        let up_sum = sum(out);

        self.filter_avg(row, up, out);
        let avg_sum = sum(out);

        self.filter_paeth(row, up, out);
        let paeth_sum = sum(out);

        let min = (none_sum, 0)
            .min((sub_sum, 1))
            .min((up_sum, 2))
            .min((avg_sum, 3))
            .min((paeth_sum, 4));

        filter_cell[0] = min.1;

        match min.1 {
            0 => self.filter_none(row, out),
            1 => self.filter_sub(row, out),
            2 => self.filter_up(row, up, out),
            3 => self.filter_avg(row, up, out),
            4 => self.filter_paeth(row, up, out),
            _ => unreachable!(),
        }
    }

    fn filter_none(&self, row: &[u8], out: &mut [u8]) {
        out[..row.len()].copy_from_slice(row);
    }

    fn filter_up(&self, row: &[u8], up: &[u8], out: &mut [u8]) {
        if up.is_empty() {
            out[..row.len()].copy_from_slice(row);
            return;
        }

        for idx in 0..row.len() {
            out[idx] = row[idx].wrapping_sub(up[idx]);
        }
    }

    fn filter_sub(&self, row: &[u8], out: &mut [u8]) {
        let mut idx = 0;

        for _ in 0..self.bpp {
            out[idx] = row[idx];
            idx += 1
        }

        for &elem in &row[self.bpp..] {
            out[idx] = elem.wrapping_sub(row[idx - self.bpp]);
            idx += 1;
        }
    }

    fn filter_avg(&self, row: &[u8], up_row: &[u8], out: &mut [u8]) {
        for idx in 0..self.bpp {
            out[idx] = row[idx].wrapping_sub(up_row[idx] / 2);
        }

        for idx in self.bpp..row.len() {
            let up = up_row[idx];
            let left = row[idx - self.bpp];

            let val = (up >> 1).wrapping_add(left >> 1) + (up & left & 0b1);

            out[idx] = row[idx].wrapping_sub(val);
        }
    }

    fn filter_paeth(&self, row: &[u8], up_row: &[u8], out: &mut [u8]) {
        for i in 0..self.bpp {
            let up = up_row[i];
            let left = 0;
            let upper_left = 0;

            let val = paeth_predictor(left, i16::from(up), upper_left);

            out[i] = row[i].wrapping_sub(val)
        }

        for i in self.bpp..row.len() {
            let up = up_row[i];
            let left = row[i - self.bpp];
            let upper_left = up_row[i - self.bpp];

            let val = paeth_predictor(i16::from(left), i16::from(up), i16::from(upper_left));

            out[i] = row[i].wrapping_sub(val)
        }
    }
}

// a = left, b = above, c = upper left
fn paeth_predictor(a: i16, b: i16, c: i16) -> u8 {
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();

    if pa <= pb && pa <= pc {
        (a % 256) as u8
    } else if pb <= pc {
        (b % 256) as u8
    } else {
        (c % 256) as u8
    }
}
