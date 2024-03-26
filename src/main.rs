use rpng::{errors::PngDecodingError, Png};

#[allow(dead_code)]
fn main() -> Result<(), PngDecodingError> {
    let png = Png::open(std::env::args().nth(1).unwrap())?;
    dbg!(&png);
    // let pixels = png.pixels()?;
    // dbg!(&pixels);

    #[cfg(feature = "serialize")]
    {
        let mut f = File::create("fogkfkg.json")?;
        f.write_all(serde_json::to_string(&pixels.rows).unwrap().as_bytes())?;
    }

    Ok(())
}
