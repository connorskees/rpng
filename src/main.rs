use std::convert::TryInto;

use rpng::{errors::PngDecodingError, Bitmap, Png, PngBuilder};

fn main() -> Result<(), PngDecodingError> {
    let png = Png::open(std::env::args().nth(1).unwrap())?;

    let resized = resize_png(&png);

    convert_to_ascii(&Bitmap {
        width: resized.width(),
        height: resized.height(),
        bpp: resized.bpp(),
        buffer: resized.idat.clone(),
    });

    png.save("./foo.png")?;

    Ok(())
}

fn convert_to_ascii(bitmap: &Bitmap) {
    let mut out = Vec::new();

    for row in bitmap.rows() {
        for channels in row.chunks_exact(4) {
            let [r, g, b, a] = TryInto::<[u8; 4]>::try_into(channels).unwrap();

            if a == 0 {
                out.push(b' ');
                continue;
            }

            let avg = (((r as u16) + (g as u16) + (b as u16)) / 3) as u8;

            if avg > 200 {
                out.push(b'#');
            } else if avg > 128 {
                out.push(b'$')
            } else if avg > 64 {
                out.push(b'*')
            } else if avg > 32 {
                out.push(b':')
            } else {
                out.push(b'.');
            }
        }
    }

    for row in out.chunks_exact(bitmap.width as usize) {
        println!("{}", std::str::from_utf8(row).unwrap());
    }
}

const RESIZE_FACTOR: i32 = 3;

fn resize_png(png: &Png) -> Png {
    let bitmap = png.decode();

    let width = png.width();

    let bpp = png.bpp();

    let new_width = png.width() as usize / RESIZE_FACTOR as usize;
    let new_height = png.height() as usize / RESIZE_FACTOR as usize;

    let mut out = vec![0; new_width * new_height * bpp];

    for y in 0..new_height {
        for x in 0..new_width {
            for bpp_offset in 0..bpp {
                out[y * new_width * bpp + x * bpp + bpp_offset] = avg_channel(
                    &bitmap,
                    x as i32,
                    y as i32,
                    bpp as i32,
                    width as i32,
                    bpp_offset as i32,
                );
            }
        }
    }

    PngBuilder::new(new_width as u32, new_height as u32)
        .color_type(png.ihdr.color_type)
        .buffer(out.clone())
        .finish()
}

fn avg_channel(bitmap: &Bitmap, x: i32, y: i32, bpp: i32, width: i32, bpp_offset: i32) -> u8 {
    let old_x = x * RESIZE_FACTOR * bpp + bpp_offset;
    let old_y = y * RESIZE_FACTOR;

    let coord = old_y * width * bpp + old_x;

    let up = coord - width * bpp;
    let down = coord + width * bpp;
    let left = coord - bpp;
    let right = coord + bpp;

    let coords = [up, down, left, right, coord];

    let coords = coords
        .iter()
        .filter(|&&c| c >= 0 && c < bitmap.buffer.len() as i32)
        .map(|&c| bitmap.buffer[c as usize] as u16);

    let count = coords.clone().count() as u16;

    let avg = coords.sum::<u16>() / count;

    assert!(avg <= u8::MAX as u16);

    avg as u8
}
