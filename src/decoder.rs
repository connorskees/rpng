use crate::{
    chunks::{
        bKGD, cHRM, gAMA, iCCP, iTXt, pHYs, sBIT, sRGB, tEXt, tRNS, AncillaryChunks, Chunk,
        UnrecognizedChunk, IHDR, PLTE,
    },
    common::{get_bit_at, ColorType, HEADER, IEND},
    errors::{ChunkError, PngDecodingError},
    interlacing, Png,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PngDecoder;

impl PngDecoder {
    pub fn read<T: std::io::BufRead + std::io::Read>(mut f: T) -> Result<Png, PngDecodingError> {
        let mut header = [0u8; 8];
        let mut ihdr: IHDR = Default::default();
        let mut unrecognized_chunks: Vec<UnrecognizedChunk> = Vec::new();
        let mut idat: Vec<u8> = Vec::new();
        let mut ancillary_chunks: AncillaryChunks = AncillaryChunks::new();
        let mut plte: Option<PLTE> = None;

        f.read_exact(&mut header)?;
        if header != HEADER {
            return Err(PngDecodingError::InvalidHeader {
                found: header,
                expected: HEADER,
            });
        }

        loop {
            let mut length_buffer: [u8; 4] = [0u8; 4];
            f.read_exact(&mut length_buffer)?;
            let length: u32 = u32::from_be_bytes(length_buffer);

            let mut chunk_type: [u8; 4] = [0; 4];
            f.read_exact(&mut chunk_type)?;

            match &chunk_type {
                // Critical
                b"IHDR" => ihdr = IHDR::parse(length, &mut f)?,
                b"PLTE" => {
                    match ihdr.color_type {
                        ColorType::Indexed | ColorType::RGB | ColorType::RGBA => {}
                        ColorType::Grayscale | ColorType::GrayscaleAlpha => {
                            return Err(ChunkError::UnexpectedPLTEChunk.into())
                        }
                    }

                    plte = Some(PLTE::parse(length, &mut f)?);
                }
                b"tRNS" => match ihdr.color_type {
                    ColorType::Grayscale => {
                        let mut grayscale_buffer = [0u8; 2];
                        f.read_exact(&mut grayscale_buffer)?;
                        let grayscale = u16::from_be_bytes(grayscale_buffer);
                        ancillary_chunks.tRNS = Some(tRNS::Grayscale { grayscale });
                    }
                    ColorType::RGB => {
                        let mut red_buffer = [0u8; 2];
                        let mut green_buffer = [0u8; 2];
                        let mut blue_buffer = [0u8; 2];

                        f.read_exact(&mut red_buffer)?;
                        f.read_exact(&mut green_buffer)?;
                        f.read_exact(&mut blue_buffer)?;

                        let red = u16::from_be_bytes(red_buffer);
                        let green = u16::from_be_bytes(green_buffer);
                        let blue = u16::from_be_bytes(blue_buffer);

                        ancillary_chunks.tRNS = Some(tRNS::RGB { red, green, blue });
                    }
                    ColorType::Indexed => {
                        let mut entries: Vec<u8> = vec![0; length as usize];
                        f.read_exact(&mut entries)?;
                        ancillary_chunks.tRNS = Some(tRNS::Indexed { entries });
                    }
                    ColorType::RGBA | ColorType::GrayscaleAlpha => todo!(),
                },
                b"IDAT" => {
                    let mut v: Vec<u8> = vec![0; length as usize];
                    f.read_exact(&mut v)?;
                    idat.extend(v);
                }
                b"IEND" => {
                    let mut iend_crc = [0u8; 4];
                    f.read_exact(&mut iend_crc)?;
                    if length != 0 || iend_crc != [174u8, 66, 96, 130] {
                        return Err(PngDecodingError::InvalidIENDChunk {
                            found: (length, iend_crc),
                            expected: IEND,
                        });
                    }
                    break;
                }

                // Ancillary
                b"pHYs" => ancillary_chunks.pHYs = Some(pHYs::parse(length, &mut f)?),
                b"tEXt" => ancillary_chunks.tEXt.push(tEXt::parse(length, &mut f)?),
                b"iTXt" => {
                    let mut keyword_buffer: Vec<u8> = Vec::new();
                    let mut compressed_buffer = [0u8];
                    let mut compression_method_buffer = [0u8];
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

                    let mut text_buffer: Vec<u8> = vec![0; remaining_length as usize];
                    f.read_exact(&mut text_buffer)?;

                    // the null byte is included in `read_until()`
                    keyword_buffer.pop();
                    language_tag_buffer.pop();
                    translated_keyword_buffer.pop();

                    let keyword = if let Ok(k) = String::from_utf8(keyword_buffer) {
                        k
                    } else {
                        continue;
                    };
                    let compressed = u8::from_be_bytes(compressed_buffer) != 0;
                    let compression_method = if compressed {
                        Some(u8::from_be_bytes(compression_method_buffer))
                    } else {
                        None
                    };
                    let language_tag = if let Ok(lt) = String::from_utf8(language_tag_buffer) {
                        lt
                    } else {
                        continue;
                    };
                    let translated_keyword =
                        if let Ok(tk) = String::from_utf8(translated_keyword_buffer) {
                            tk
                        } else {
                            continue;
                        };
                    let text = if compressed {
                        todo!()
                    } else if let Ok(t) = String::from_utf8(text_buffer) {
                        t
                    } else {
                        continue;
                    };

                    let itxt = iTXt {
                        keyword,
                        compressed,
                        compression_method,
                        language_tag,
                        translated_keyword,
                        text,
                    };
                    ancillary_chunks.itxt.push(itxt);
                }
                b"bKGD" => match ihdr.color_type {
                    ColorType::Grayscale | ColorType::GrayscaleAlpha => {
                        let mut grayscale_buffer = [0u8; 2];
                        f.read_exact(&mut grayscale_buffer)?;
                        let grayscale = u16::from_be_bytes(grayscale_buffer);
                        ancillary_chunks.bKGD = Some(bKGD::Grayscale { grayscale });
                    }
                    ColorType::RGB | ColorType::RGBA => {
                        let mut red_buffer = [0u8; 2];
                        let mut green_buffer = [0u8; 2];
                        let mut blue_buffer = [0u8; 2];
                        f.read_exact(&mut red_buffer)?;
                        f.read_exact(&mut green_buffer)?;
                        f.read_exact(&mut blue_buffer)?;
                        let red = u16::from_be_bytes(red_buffer);
                        let green = u16::from_be_bytes(green_buffer);
                        let blue = u16::from_be_bytes(blue_buffer);
                        ancillary_chunks.bKGD = Some(bKGD::RGB { red, green, blue });
                    }
                    ColorType::Indexed => {
                        let mut palette_index_buffer = [0u8];
                        f.read_exact(&mut palette_index_buffer)?;
                        let palette_index = u8::from_be_bytes(palette_index_buffer);
                        let rgb = if let Some(p) = plte.clone() {
                            p
                        } else {
                            unreachable!()
                        };
                        ancillary_chunks.bKGD = Some(bKGD::Palette {
                            palette_index,
                            rgb: rgb[palette_index],
                        });
                    }
                },
                b"gAMA" => {
                    if length != 4 {
                        return Err(ChunkError::InvalidgAMALength.into());
                    }
                    let mut gamma_buffer = [0u8; 4];
                    f.read_exact(&mut gamma_buffer)?;
                    let gamma = u32::from_be_bytes(gamma_buffer);
                    ancillary_chunks.gama = Some(gAMA { gamma });
                }
                b"cHRM" => ancillary_chunks.chrm = Some(cHRM::parse(length, &mut f)?),
                b"iCCP" => ancillary_chunks.iCCP = Some(iCCP::parse(length, &mut f)?),
                b"sBIT" => {
                    ancillary_chunks.sBIT = match ihdr.color_type {
                        ColorType::Grayscale => {
                            let mut grayscale_buffer = [0];
                            f.read_exact(&mut grayscale_buffer)?;
                            let grayscale = u8::from_be_bytes(grayscale_buffer);
                            Some(sBIT::Grayscale { grayscale })
                        }
                        ColorType::RGB => {
                            let mut red_buffer = [0u8];
                            let mut green_buffer = [0u8];
                            let mut blue_buffer = [0u8];

                            f.read_exact(&mut red_buffer)?;
                            f.read_exact(&mut green_buffer)?;
                            f.read_exact(&mut blue_buffer)?;

                            let red = u8::from_be_bytes(red_buffer);
                            let green = u8::from_be_bytes(green_buffer);
                            let blue = u8::from_be_bytes(blue_buffer);

                            Some(sBIT::RGB { red, green, blue })
                        }
                        ColorType::Indexed => {
                            let mut red_buffer = [0u8];
                            let mut green_buffer = [0u8];
                            let mut blue_buffer = [0u8];

                            f.read_exact(&mut red_buffer)?;
                            f.read_exact(&mut green_buffer)?;
                            f.read_exact(&mut blue_buffer)?;

                            let red = u8::from_be_bytes(red_buffer);
                            let green = u8::from_be_bytes(green_buffer);
                            let blue = u8::from_be_bytes(blue_buffer);

                            Some(sBIT::Indexed { red, green, blue })
                        }
                        ColorType::GrayscaleAlpha => {
                            let mut grayscale_buffer = [0u8];
                            let mut alpha_buffer = [0u8];

                            f.read_exact(&mut grayscale_buffer)?;
                            f.read_exact(&mut alpha_buffer)?;

                            let grayscale = u8::from_be_bytes(grayscale_buffer);
                            let alpha = u8::from_be_bytes(alpha_buffer);

                            Some(sBIT::GrayscaleAlpha { grayscale, alpha })
                        }
                        ColorType::RGBA => {
                            let mut red_buffer = [0u8];
                            let mut green_buffer = [0u8];
                            let mut blue_buffer = [0u8];
                            let mut alpha_buffer = [0u8];

                            f.read_exact(&mut red_buffer)?;
                            f.read_exact(&mut green_buffer)?;
                            f.read_exact(&mut blue_buffer)?;
                            f.read_exact(&mut alpha_buffer)?;

                            let red = u8::from_be_bytes(red_buffer);
                            let green = u8::from_be_bytes(green_buffer);
                            let blue = u8::from_be_bytes(blue_buffer);
                            let alpha = u8::from_be_bytes(alpha_buffer);

                            Some(sBIT::RGBA {
                                red,
                                green,
                                blue,
                                alpha,
                            })
                        }
                    }
                }
                b"sRGB" => {
                    let mut intent_buffer = [0];
                    f.read_exact(&mut intent_buffer)?;

                    ancillary_chunks.sRGB = Some(sRGB::from_u8(u8::from_be_bytes(intent_buffer))?);
                }
                _ => {
                    let is_critical = !get_bit_at(chunk_type[0], 5);
                    let is_public = !get_bit_at(chunk_type[1], 5);
                    let is_safe_to_copy = get_bit_at(chunk_type[2], 5);
                    if is_critical {
                        return Err(ChunkError::UnrecognizedCriticalChunk(chunk_type).into());
                    }
                    let mut buffer: Vec<u8> = vec![0; length as usize];
                    f.read_exact(&mut buffer)?;
                    unrecognized_chunks.push(UnrecognizedChunk {
                        length,
                        chunk_type,
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

        idat = match ihdr.interlace_method {
            // adam7
            1 => interlacing::decode_adam7(&idat),
            // no interlacing or invalid value
            _ => idat,
        };

        Ok(Png {
            ihdr,
            idat,
            decoded_buffer: None,
            unrecognized_chunks,
            ancillary_chunks,
            plte,
        })
    }
}
