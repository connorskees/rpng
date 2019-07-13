#![allow(dead_code)]
#![deny(unsafe_code)]

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::{fmt, fs, str};
use std::vec::Vec;

use flate2::bufread::ZlibDecoder;
// use serde_json;

use chunks::{IHDR, Chunk, pHYs, iTXt, AncillaryChunks};
use common::{ColorType, Unit};

mod common;
mod chunks;
mod filter;

const FILE_NAME: &str = "redrect";

struct PNG {
    ihdr: IHDR,
    idat: Vec<u8>,
    unrecognized_chunks: Vec<Chunk>,
    ancillary_chunks: AncillaryChunks,
}

impl fmt::Debug for PNG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "PNG {{\n    ihdr: {:?}\n    idat: {} bytes (compressed)\n    unrecognized_chunks: {:#?}\n    ancillary_chunks: {:#?}}}",
            self.ihdr, self.idat.len(), self.unrecognized_chunks, self.ancillary_chunks
        )
    }
}

impl PNG {
    pub fn from_path(file_path: &str) -> io::Result<Self> {
        let metadata = fs::metadata(file_path)?;
        PNG::from(BufReader::with_capacity(metadata.len() as usize, File::open(file_path)?))
    }

    pub fn from<T: std::io::BufRead + std::io::Read>(mut f: T) -> io::Result<Self> {
        let mut header = [0; 8];
        let mut ihdr: IHDR = Default::default();
        let mut unrecognized_chunks: Vec<Chunk> = Vec::new();
        let mut idat: Vec<u8> = Vec::new();
        let mut ancillary_chunks: AncillaryChunks = AncillaryChunks::new();

        f.read(&mut header)?;
        if header != [137u8, 80, 78, 71, 13, 10, 26, 10] {
            panic!("invalid header");
        }

        loop {
            let mut length_buffer: [u8; 4] = [0; 4];
            f.read(&mut length_buffer)?;
            let length: u32 = u32::from_be_bytes(length_buffer);

            let mut type_buffer: [u8; 4] = [0; 4];
            f.read(&mut type_buffer)?;
            let chunk_type = str::from_utf8(&type_buffer).unwrap();
            println!("{:#?}", chunk_type);



            match chunk_type {
                // Critical
                "IHDR" => {
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
                        panic!("invalid IHDR length");
                    }

                    f.read(&mut width_buffer)?;
                    ihdr.width = u32::from_be_bytes(width_buffer);
                    
                    f.read(&mut height_buffer)?;
                    ihdr.height = u32::from_be_bytes(height_buffer);
                    
                    f.read(&mut bit_depth_buffer)?;
                    ihdr.bit_depth = u8::from_be_bytes(bit_depth_buffer);
                    
                    f.read(&mut color_type_buffer)?;
                    ihdr.color_type = ColorType::from_u8(u8::from_be_bytes(color_type_buffer));
                    
                    f.read(&mut compression_type_buffer)?;
                    ihdr.compression_type = u8::from_be_bytes(compression_type_buffer);
                    
                    f.read(&mut filter_method_buffer)?;
                    ihdr.filter_method = u8::from_be_bytes(filter_method_buffer);
                    
                    f.read(&mut interlace_method_buffer)?;
                    ihdr.interlace_method = u8::from_be_bytes(interlace_method_buffer);
                },
                "PLTE" => {
                    // if length % 3 != 0 {
                    //     panic!("PLTE chunk length must be divisible by 3")
                    // }
                    // println!("fjghjfhjfh")
                },
                "IDAT" => {
                    let mut v: Vec<u8> = vec!(0; length as usize);
                    f.read(&mut v)?;
                    idat.extend(v);
                },
                "IEND" => {
                    break;
                },

                // Ancillary
                "pHYs" => {
                    let mut pixels_per_x_buffer = [0; 4];
                    let mut pixels_per_y_buffer = [0; 4];
                    let mut unit_buffer = [0];

                    f.read(&mut pixels_per_x_buffer)?;
                    let pixels_per_unit_x = u32::from_be_bytes(pixels_per_x_buffer);

                    f.read(&mut pixels_per_y_buffer)?;
                    let pixels_per_unit_y = u32::from_be_bytes(pixels_per_y_buffer);

                    f.read(&mut unit_buffer)?;
                    let unit = u8::from_be_bytes(unit_buffer);

                    ancillary_chunks.phys = Some(pHYs {
                        pixels_per_unit_x, pixels_per_unit_y,
                        unit: Unit::from_u8(unit)
                    });
                },
                "iTXt" => {
                    let mut keyword_buffer = Vec::new();
                    let mut compressed_buffer = [0];
                    let mut compression_method_buffer = [0];
                    let mut language_tag_buffer = Vec::new();
                    let mut translated_keyword_buffer = Vec::new();

                    let keyword_len = f.read_until(b'\0', &mut keyword_buffer)?;
                    f.read(&mut compressed_buffer)?;
                    f.read(&mut compression_method_buffer)?;
                    let language_tag_len = f.read_until(0, &mut language_tag_buffer)?;
                    let translated_keyword_len = f.read_until(0, &mut translated_keyword_buffer)?;

                    let remaining_length = length
                                            - (keyword_len as u32)
                                            - 2
                                            - (language_tag_len as u32)
                                            - (translated_keyword_len as u32);
                    
                    let mut text_buffer: Vec<u8> = vec!(0; remaining_length as usize);
                    f.read(&mut text_buffer)?;

                    println!("{:?}", keyword_buffer);

                    let keyword = String::from_utf8(keyword_buffer).unwrap();
                    let compressed = u8::from_be_bytes(compressed_buffer) != 0;
                    let compression_method = u8::from_be_bytes(compression_method_buffer);
                    let language_tag = String::from_utf8(language_tag_buffer).unwrap();
                    let translated_keyword = String::from_utf8(translated_keyword_buffer).unwrap();
                    let text = String::from_utf8(text_buffer).unwrap();

                    let itxt = iTXt {
                        keyword,
                        compressed,
                        compression_method,
                        language_tag,
                        translated_keyword,
                        text,
                    };
                    ancillary_chunks.itxt.push(Some(itxt));
                },
                _ => {
                    let mut v: Vec<u8> = vec!(0; length as usize);
                    f.read(&mut v)?;
                    unrecognized_chunks.push(Chunk {
                        length,
                        chunk_type: String::from(chunk_type),
                        bytes: v,
                    })
                }
            }

            let mut crc = [0; 4];
            f.read_exact(&mut crc)?;
        }

        ihdr.validate_fields().unwrap();

        Ok(PNG {
            ihdr,
            idat,
            unrecognized_chunks,
            ancillary_chunks,
        })
    }

    pub fn pixels(&self) -> io::Result<Vec<Vec<Vec<u8>>>> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut zlib = ZlibDecoder::new(&self.idat as &[u8]);
        zlib.read_to_end(&mut buffer)?;

        let mut rows: Vec<Vec<Vec<u8>>> = Vec::new();
        match self.ihdr.color_type {
            ColorType::RGBA => {
                let filtered_rows: Vec<&[u8]> = buffer.chunks((1+self.ihdr.width*4) as usize).collect::<Vec<_>>();
                for (idx, row) in filtered_rows.iter().enumerate() {
                    rows.push(match row[0] {
                        0 => row.chunks(4).map(|x| Vec::from(x)).collect(),
                        1 => filter::sub(&row[1..], 4, true),
                        2 => filter::up(&row[1..], &rows[idx-1], 4, true),
                        3 => filter::average(&row[1..], &rows[idx-1], 4),
                        4 => filter::paeth(&row[1..], &rows[idx-1], 4, true),
                        _ => row.chunks(4).map(|x| Vec::from(x)).collect(),
                    });
                }
                // println!("{:?}", rows);
            }
            _ => {
                panic!("invalid color type");
            }
        }
        Ok(rows)
    }
}

fn main() -> io::Result<()> {
    let png = PNG::from_path(&format!("pngs/{}.png", FILE_NAME))?;
    // let png = PNG::from_path(r"C:\Users\Connor\Downloads\PngSuite-2017jul19\oi9n2c16.png")?;
    println!("{:?}", png);
    let pixels = png.pixels()?;
    let mut f = File::create("fogkfkg.json")?;
    f.write(serde_json::to_string(&pixels)?.as_bytes())?;
    // println!("\n{:?}", pixels[0][0]);
    Ok(())
}
// 78 9C
// ---CMF---  ---FLG---
// 0111.1000  1001.1100

// 01 ff 00 00 ff 00 00 00 00 00 00 00 00 01 00 00
// 00 00 00 00 00 00 00 00 00 00 00 00 00 02 00 00
// 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
// 00 00 00 00 00 00 00 00 00 00 02 00 00 00 00 00
// 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
// 00 00 00 00 00 00 00 01 00 00 ff ff 00 00 00 00
// 00 00 00 00 00 ff 01 00 00 00 00 00 00 00 00 00
// 00 00 00 00 02 00 00 00 00 00 00 00 00 00 00 00
// 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
// 00 02 00 00 00 00 00 00 00 00 00 00 00 00 00 00
// 00 00 00 00 00 00 00 00 00 00 00 00 00 00