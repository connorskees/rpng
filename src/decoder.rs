use std::{str};
use std::vec::Vec;

use crate::chunks::{IHDR, PLTE, UnrecognizedChunk, pHYs, iTXt, gAMA, cHRM, iCCP, PaletteEntry, AncillaryChunks};
use crate::common::{get_bit_at, BitDepth, ColorType, CompressionType, Unit};
use crate::filter::{FilterMethod};
use crate::interlacing::{Interlacing};
use crate::errors::PNGDecodingError;
use crate::PNG;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PNGDecoder;

impl PNGDecoder {
    pub fn read<T: std::io::BufRead + std::io::Read>(mut f: T) -> Result<PNG, PNGDecodingError> {
        let mut header = [0; 8];
        let mut ihdr: IHDR = Default::default();
        let mut unrecognized_chunks: Vec<UnrecognizedChunk> = Vec::new();
        let mut idat: Vec<u8> = Vec::new();
        let mut ancillary_chunks: AncillaryChunks = AncillaryChunks::new();
        let mut plte: Option<PLTE> = None;

        f.read_exact(&mut header)?;
        if header != [137u8, 80, 78, 71, 13, 10, 26, 10] {
            return Err(PNGDecodingError::InvalidHeader(header, "expected [137, 80, 78, 71, 13, 10, 26, 10]"));
        }

        loop {
            let mut length_buffer: [u8; 4] = [0; 4];
            f.read_exact(&mut length_buffer)?;
            let length: u32 = u32::from_be_bytes(length_buffer);

            let mut chunk_type_buffer: [u8; 4] = [0; 4];
            f.read_exact(&mut chunk_type_buffer)?;
            let chunk_type = str::from_utf8(&chunk_type_buffer)?;
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
                        return Err(PNGDecodingError::InvalidIHDRLength(length));
                    }

                    f.read_exact(&mut width_buffer)?;
                    let width = u32::from_be_bytes(width_buffer);
                    
                    f.read_exact(&mut height_buffer)?;
                    let height = u32::from_be_bytes(height_buffer);
                    
                    f.read_exact(&mut bit_depth_buffer)?;
                    let bit_depth = BitDepth::from_u8(u8::from_be_bytes(bit_depth_buffer))?;
                    
                    f.read_exact(&mut color_type_buffer)?;
                    let color_type = ColorType::from_u8(u8::from_be_bytes(color_type_buffer))?;
                    
                    f.read_exact(&mut compression_type_buffer)?;
                    let compression_type = CompressionType::from_u8(u8::from_be_bytes(compression_type_buffer))?;
                    
                    f.read_exact(&mut filter_method_buffer)?;
                    let filter_method = FilterMethod::from_u8(u8::from_be_bytes(filter_method_buffer))?;
                    
                    f.read_exact(&mut interlace_method_buffer)?;
                    let interlace_method = Interlacing::from_u8(u8::from_be_bytes(interlace_method_buffer));

                    ihdr = IHDR::new(width, height, bit_depth, color_type, compression_type, filter_method, interlace_method)?;
                },
                "PLTE" => {
                    if length % 3 != 0 {
                        return Err(PNGDecodingError::InvalidPLTELength)
                    }
                    match ihdr.color_type {
                        ColorType::Indexed | ColorType::RGB | ColorType::RGBA => {},
                        ColorType::Grayscale | ColorType:: GrayscaleAlpha => return Err(PNGDecodingError::UnexpectedPLTEChunk),
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
                        unit: Unit::from_u8(unit)?
                    });
                },
                "iTXt" => {
                    let mut keyword_buffer: Vec<u8> = Vec::new();
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

                    let keyword = if let Ok(k) = String::from_utf8(keyword_buffer) { k } else { continue };
                    let compressed = u8::from_be_bytes(compressed_buffer) != 0;
                    let compression_method = if compressed { Some(CompressionType::from_u8(u8::from_be_bytes(compression_method_buffer))?) } else { None };
                    let language_tag = if let Ok(lt) = String::from_utf8(language_tag_buffer) { lt } else { continue };
                    let translated_keyword = if let Ok(tk) = String::from_utf8(translated_keyword_buffer) { tk } else { continue };
                    let text = if let Ok(t) = String::from_utf8(text_buffer) { t } else { continue };

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
                    let mut profile_name_buffer: Vec<u8> = Vec::new();
                    let mut compression_method_buffer = [0];

                    let profile_name_len = f.read_until(b'\0', &mut profile_name_buffer)?;
                    f.read_exact(&mut compression_method_buffer)?;

                    let remaining_length = length
                                            - (profile_name_len as u32)
                                            - 1;
                    
                    let mut compressed_profile: Vec<u8> = vec!(0; remaining_length as usize);
                    f.read_exact(&mut compressed_profile)?;

                    let profile_name = if let Ok(pn) = String::from_utf8(profile_name_buffer) { pn } else { continue };
                    let compression_method = CompressionType::from_u8(u8::from_be_bytes(compression_method_buffer))?;
                    ancillary_chunks.iccp = Some(iCCP {
                        profile_name, compression_method, compressed_profile,
                    });
                }
                _ => {
                    let mut buffer: Vec<u8> = vec!(0; length as usize);
                    f.read_exact(&mut buffer)?;
                    let is_critical = get_bit_at(chunk_type_buffer[0], 5).unwrap() == 0;
                    let is_public = get_bit_at(chunk_type_buffer[1], 5).unwrap() == 0;
                    let is_safe_to_copy = get_bit_at(chunk_type_buffer[2], 5).unwrap() == 1;
                    if is_critical {
                        panic!("unrecognized critical chunk found");
                    }
                    unrecognized_chunks.push(UnrecognizedChunk {
                        length,
                        chunk_type: String::from(chunk_type),
                        bytes: buffer,
                        is_critical,
                        is_public,
                        is_safe_to_copy,
                    })
                }
            }

            let mut crc = [0; 4];
            f.read_exact(&mut crc)?;
        }

        // idat = match ihdr.interlace_method {
        //     Interlacing::None => idat,
        //     Interlacing::Adam7 => Interlacing::adam7(idat)
        // };

        Ok(PNG {
            ihdr,
            idat,
            unrecognized_chunks,
            ancillary_chunks,
            plte
        })
    }
}