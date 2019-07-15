//! Module for working with PNG files

#![allow(dead_code)]
#![deny(unsafe_code)]

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::{fmt, fs, str};
use std::vec::Vec;

use flate2::bufread::ZlibDecoder;
// use serde_json;

use chunks::{IHDR, PLTE, UnrecognizedChunk, pHYs, iTXt, gAMA, cHRM, iCCP, PaletteEntry, AncillaryChunks};
pub use common::{BitDepth, ColorType, CompressionType, Unit, Interlacing};
pub use filter::{FilterMethod, FilterType};

mod common;
pub mod chunks;
mod filter;

const FILE_NAME: &str = "redrect";

struct PNG {
    ihdr: IHDR,
    plte: Option<PLTE>,
    idat: Vec<u8>,
    unrecognized_chunks: Vec<UnrecognizedChunk>,
    ancillary_chunks: AncillaryChunks,
}

impl fmt::Debug for PNG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "PNG {{\n    ihdr: {:?}\n    plte: {:?}\n    data: {} bytes (compressed)\n    unrecognized_chunks: {:#?}\n    ancillary_chunks: {:#?}}}",
            self.ihdr, self.plte.as_ref(), self.idat.len(), self.unrecognized_chunks, self.ancillary_chunks
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
        let mut unrecognized_chunks: Vec<UnrecognizedChunk> = Vec::new();
        let mut idat: Vec<u8> = Vec::new();
        let mut ancillary_chunks: AncillaryChunks = AncillaryChunks::new();
        let mut plte: Option<PLTE> = None;

        f.read_exact(&mut header)?;
        if header != [137u8, 80, 78, 71, 13, 10, 26, 10] {
            panic!("invalid header");
        }

        loop {
            let mut length_buffer: [u8; 4] = [0; 4];
            f.read_exact(&mut length_buffer)?;
            let length: u32 = u32::from_be_bytes(length_buffer);

            let mut chunk_type_buffer: [u8; 4] = [0; 4];
            f.read_exact(&mut chunk_type_buffer)?;
            let chunk_type = str::from_utf8(&chunk_type_buffer).unwrap();
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

                    f.read_exact(&mut width_buffer)?;
                    ihdr.width = u32::from_be_bytes(width_buffer);
                    
                    f.read_exact(&mut height_buffer)?;
                    ihdr.height = u32::from_be_bytes(height_buffer);
                    
                    f.read_exact(&mut bit_depth_buffer)?;
                    ihdr.bit_depth = BitDepth::from_u8(u8::from_be_bytes(bit_depth_buffer));
                    
                    f.read_exact(&mut color_type_buffer)?;
                    ihdr.color_type = ColorType::from_u8(u8::from_be_bytes(color_type_buffer));
                    
                    f.read_exact(&mut compression_type_buffer)?;
                    ihdr.compression_type = CompressionType::from_u8(u8::from_be_bytes(compression_type_buffer));
                    
                    f.read_exact(&mut filter_method_buffer)?;
                    ihdr.filter_method = FilterMethod::from_u8(u8::from_be_bytes(filter_method_buffer));
                    
                    f.read_exact(&mut interlace_method_buffer)?;
                    ihdr.interlace_method = Interlacing::from_u8(u8::from_be_bytes(interlace_method_buffer));
                },
                "PLTE" => {
                    if length % 3 != 0 {
                        panic!("PLTE chunk length must be divisible by 3")
                    }
                    match ihdr.color_type {
                        ColorType::Indexed | ColorType::RGB | ColorType::RGBA => {},
                        ColorType::Grayscale | ColorType:: GrayscaleAlpha => panic!("unexpected PLTE chunk")   
                    }
                    let mut entries_buffer: Vec<u8> = vec!(0; length as usize);
                    f.read_exact(&mut entries_buffer)?;
                    let entries_: Vec<&[u8]> = entries_buffer.chunks(3).collect();
                    let entries: Vec<PaletteEntry> =  entries_.iter().map(|x| PaletteEntry::from_u8(x)).collect();

                    plte = Some(PLTE {
                        entries
                    });
                },
                "IDAT" => {
                    let mut v: Vec<u8> = vec!(0; length as usize);
                    f.read_exact(&mut v)?;
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

                    f.read_exact(&mut pixels_per_x_buffer)?;
                    let pixels_per_unit_x = u32::from_be_bytes(pixels_per_x_buffer);

                    f.read_exact(&mut pixels_per_y_buffer)?;
                    let pixels_per_unit_y = u32::from_be_bytes(pixels_per_y_buffer);

                    f.read_exact(&mut unit_buffer)?;
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
                    f.read_exact(&mut compressed_buffer)?;
                    f.read_exact(&mut compression_method_buffer)?;
                    let language_tag_len = f.read_until(0, &mut language_tag_buffer)?;
                    let translated_keyword_len = f.read_until(0, &mut translated_keyword_buffer)?;

                    let remaining_length = length
                                            - (keyword_len as u32)
                                            - 2
                                            - (language_tag_len as u32)
                                            - (translated_keyword_len as u32);
                    
                    let mut text_buffer: Vec<u8> = vec!(0; remaining_length as usize);
                    f.read_exact(&mut text_buffer)?;

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
                "gAMA" => {
                    if length != 4 {
                        panic!("invalid gAMA length");
                    }
                    let mut gamma_buffer = [0; 4];
                    f.read_exact(&mut gamma_buffer)?;
                    let gamma = u32::from_be_bytes(gamma_buffer);
                    ancillary_chunks.gama = Some(gAMA { gamma });
                },
                "cHRM" => {
                    let (
                        mut white_point_x_buffer,
                        mut white_point_y_buffer,
                        mut red_x_buffer,
                        mut red_y_buffer,
                        mut green_x_buffer,
                        mut green_y_buffer,
                        mut blue_x_buffer,
                        mut blue_y_buffer
                    ) = ([0; 4], [0; 4], [0; 4], [0; 4], [0; 4], [0; 4], [0; 4], [0; 4]);
                    
                    f.read_exact(&mut white_point_x_buffer)?;
                    let white_point_x = u32::from_be_bytes(white_point_x_buffer);

                    f.read_exact(&mut white_point_y_buffer)?;
                    let white_point_y = u32::from_be_bytes(white_point_y_buffer);

                    f.read_exact(&mut red_x_buffer)?;
                    let red_x = u32::from_be_bytes(red_x_buffer);

                    f.read_exact(&mut red_y_buffer)?;
                    let red_y = u32::from_be_bytes(red_y_buffer);

                    f.read_exact(&mut green_x_buffer)?;
                    let green_x = u32::from_be_bytes(green_x_buffer);

                    f.read_exact(&mut green_y_buffer)?;
                    let green_y = u32::from_be_bytes(green_y_buffer);

                    f.read_exact(&mut blue_x_buffer)?;
                    let blue_x = u32::from_be_bytes(blue_x_buffer);

                    f.read_exact(&mut blue_y_buffer)?;
                    let blue_y = u32::from_be_bytes(blue_y_buffer);

                    ancillary_chunks.chrm = Some(cHRM {
                        white_point_x,
                        white_point_y,
                        red_x,
                        red_y,
                        green_x,
                        green_y,
                        blue_x,
                        blue_y, 
                    });
                },
                "iCCP" => {

                }
                _ => {
                    let mut v: Vec<u8> = vec!(0; length as usize);
                    f.read(&mut v)?;
                    let is_critical = common::get_bit_at(v[0], 5).unwrap() == 0;
                    let is_public = common::get_bit_at(v[1], 5).unwrap() == 0;
                    let is_safe_to_copy = common::get_bit_at(v[2], 5).unwrap() == 1;
                    if is_critical {
                        panic!("unrecognized critical chunk found");
                    }
                    unrecognized_chunks.push(UnrecognizedChunk {
                        length,
                        chunk_type: String::from(chunk_type),
                        bytes: v,
                        is_critical,
                        is_public,
                        is_safe_to_copy,
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
            plte
        })
    }

    pub fn pixels(&self) -> io::Result<Vec<Vec<Vec<u8>>>> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut zlib = ZlibDecoder::new(&self.idat as &[u8]);
        zlib.read_to_end(&mut buffer)?;

        let mut rows: Vec<Vec<Vec<u8>>> = Vec::new();
        let chunk_length: u8 = match self.ihdr.color_type {
            ColorType::Grayscale => 1,
            ColorType::RGB => 3,
            ColorType::Indexed => 1,
            ColorType:: GrayscaleAlpha => 2,
            ColorType::RGBA => 4,
        };
        if self.ihdr.bit_depth.as_u8() < 8 {

        }
        println!("raw buf len {:?}", buffer.len());
        let row_length = 1 + (((self.ihdr.bit_depth.as_u8() as f32/8f32) * self.ihdr.width as f32).ceil() as u32 * (chunk_length as u32));
        println!("row length {}", row_length);
        let filtered_rows: Vec<&[u8]> = buffer.chunks(row_length as usize).collect::<Vec<&[u8]>>();
        // println!("buffer {:?}", filtered_rows);
        println!("num of rows {}", filtered_rows.len());
        // println!("{:?}", filtered_rows.len());
        for (idx, row) in filtered_rows.iter().enumerate() {
            // println!("{:?}", row);
            rows.push(match FilterType::from_u8(row[0]) {
                FilterType::None => row[1..].chunks(chunk_length as usize).map(|x| Vec::from(x)).collect(),
                FilterType::Sub => filter::sub(&row[1..], chunk_length, true),
                FilterType::Up => filter::up(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length, true),
                FilterType::Average => filter::average(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length),
                FilterType::Paeth => filter::paeth(&row[1..], if idx == 0 { None } else { Some(&rows[idx-1]) }, chunk_length, true),
            });
        }
        // println!("rows {:?}", rows);
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
