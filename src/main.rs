// use std::convert::TryInto;

use rpng::{errors::PngDecodingError, Png};

#[allow(dead_code)]
fn main() -> Result<(), PngDecodingError> {
    let png = Png::open(std::env::args().nth(1).unwrap())?;
    // let bitmap = png.decode();

    // for row in bitmap.rows() {
    //     for channels in row.chunks_exact(4) {
    //         let [r, g, b, a] = TryInto::<[u8; 4]>::try_into(channels).unwrap();

    //         let avg = (((r as u16) + (g as u16) + (b as u16)) / 3) as u8;

    //     }
    // }

    png.save("./foo.png")?;

    #[cfg(feature = "serialize")]
    {
        let mut f = File::create("fogkfkg.json")?;
        f.write_all(serde_json::to_string(&pixels.rows).unwrap().as_bytes())?;
    }

    Ok(())
}
