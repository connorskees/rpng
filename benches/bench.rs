#![feature(test)]
#![allow(dead_code, unused_imports, unused_macros)]

use std::path::Path;

extern crate test;
use test::Bencher;

/// File naming conventions:
/// Tests:
///     basi - basic tests
///     
/// color-type:
///     0g - grayscale
///     2c - rgb color
///     3p - paletted
///     4a - grayscale + alpha channel
///     6a - rgb color + alpha channel
/// bit-depth:
///     01 - with color-type 0, 3
///     02 - with color-type 0, 3
///     04 - with color-type 0, 3
///     08 - with color-type 0, 2, 3, 4, 6
///     16 - with color-type 0, 2, 4, 6
/// interlacing:
///     n - non-interlaced
///     i - interlaced
macro_rules! pngsuite {
    ($func:ident) => {
        $func!(basi0g01, "pngsuite");
        $func!(basi0g02, "pngsuite");
        $func!(basi0g04, "pngsuite");
        $func!(basi0g08, "pngsuite");
        $func!(basi0g16, "pngsuite");
        $func!(basi2c08, "pngsuite");
        $func!(basi2c16, "pngsuite");
        $func!(basi3p01, "pngsuite");
        $func!(basi3p02, "pngsuite");
        $func!(basi3p04, "pngsuite");
        $func!(basi3p08, "pngsuite");
        $func!(basi4a08, "pngsuite");
        $func!(basi4a16, "pngsuite");
        $func!(basi6a08, "pngsuite");
        $func!(basi6a16, "pngsuite");
        $func!(basn0g01, "pngsuite");
        $func!(basn0g02, "pngsuite");
        $func!(basn0g04, "pngsuite");
        $func!(basn0g08, "pngsuite");
        $func!(basn0g16, "pngsuite");
        $func!(basn2c08, "pngsuite");
        $func!(basn2c16, "pngsuite");
        $func!(basn3p01, "pngsuite");
        $func!(basn3p02, "pngsuite");
        $func!(basn3p04, "pngsuite");
        $func!(basn3p08, "pngsuite");
        $func!(basn4a08, "pngsuite");
        $func!(basn4a16, "pngsuite");
        $func!(basn6a08, "pngsuite");
        $func!(basn6a16, "pngsuite");
        $func!(bgai4a08, "pngsuite");
        $func!(bgai4a16, "pngsuite");
        $func!(bgan6a08, "pngsuite");
        $func!(bgan6a16, "pngsuite");
        $func!(bgbn4a08, "pngsuite");
        $func!(bggn4a16, "pngsuite");
        $func!(bgwn6a08, "pngsuite");
        $func!(bgyn6a16, "pngsuite");
        $func!(ccwn2c08, "pngsuite");
        $func!(ccwn3p08, "pngsuite");
        $func!(cdfn2c08, "pngsuite");
        $func!(cdhn2c08, "pngsuite");
        $func!(cdsn2c08, "pngsuite");
        $func!(cdun2c08, "pngsuite");
        $func!(ch1n3p04, "pngsuite");
        $func!(ch2n3p08, "pngsuite");
        $func!(cm0n0g04, "pngsuite");
        $func!(cm7n0g04, "pngsuite");
        $func!(cm9n0g04, "pngsuite");
        $func!(cs3n2c16, "pngsuite");
        $func!(cs3n3p08, "pngsuite");
        $func!(cs5n2c08, "pngsuite");
        $func!(cs5n3p08, "pngsuite");
        $func!(cs8n2c08, "pngsuite");
        $func!(cs8n3p08, "pngsuite");
        $func!(ct0n0g04, "pngsuite");
        $func!(ct1n0g04, "pngsuite");
        $func!(cten0g04, "pngsuite");
        $func!(ctfn0g04, "pngsuite");
        $func!(ctgn0g04, "pngsuite");
        $func!(cthn0g04, "pngsuite");
        $func!(ctjn0g04, "pngsuite");
        $func!(ctzn0g04, "pngsuite");
        $func!(exif2c08, "pngsuite");
        $func!(f00n0g08, "pngsuite");
        $func!(f00n2c08, "pngsuite");
        $func!(f01n0g08, "pngsuite");
        $func!(f01n2c08, "pngsuite");
        $func!(f02n0g08, "pngsuite");
        $func!(f02n2c08, "pngsuite");
        $func!(f03n0g08, "pngsuite");
        $func!(f03n2c08, "pngsuite");
        $func!(f04n0g08, "pngsuite");
        $func!(f04n2c08, "pngsuite");
        $func!(f99n0g04, "pngsuite");
        $func!(g03n0g16, "pngsuite");
        $func!(g03n2c08, "pngsuite");
        $func!(g03n3p04, "pngsuite");
        $func!(g04n0g16, "pngsuite");
        $func!(g04n2c08, "pngsuite");
        $func!(g04n3p04, "pngsuite");
        $func!(g05n0g16, "pngsuite");
        $func!(g05n2c08, "pngsuite");
        $func!(g05n3p04, "pngsuite");
        $func!(g07n0g16, "pngsuite");
        $func!(g07n2c08, "pngsuite");
        $func!(g07n3p04, "pngsuite");
        $func!(g10n0g16, "pngsuite");
        $func!(g10n2c08, "pngsuite");
        $func!(g10n3p04, "pngsuite");
        $func!(g25n0g16, "pngsuite");
        $func!(g25n2c08, "pngsuite");
        $func!(g25n3p04, "pngsuite");
        $func!(oi1n0g16, "pngsuite");
        $func!(oi1n2c16, "pngsuite");
        $func!(oi2n0g16, "pngsuite");
        $func!(oi2n2c16, "pngsuite");
        $func!(oi4n0g16, "pngsuite");
        $func!(oi4n2c16, "pngsuite");
        $func!(oi9n0g16, "pngsuite");
        $func!(oi9n2c16, "pngsuite");
        $func!(pp0n2c16, "pngsuite");
        $func!(pp0n6a08, "pngsuite");
        $func!(ps1n0g08, "pngsuite");
        $func!(ps1n2c16, "pngsuite");
        $func!(ps2n0g08, "pngsuite");
        $func!(ps2n2c16, "pngsuite");
        $func!(s01i3p01, "pngsuite");
        $func!(s01n3p01, "pngsuite");
        $func!(s02i3p01, "pngsuite");
        $func!(s02n3p01, "pngsuite");
        $func!(s03i3p01, "pngsuite");
        $func!(s03n3p01, "pngsuite");
        $func!(s04i3p01, "pngsuite");
        $func!(s04n3p01, "pngsuite");
        $func!(s05i3p02, "pngsuite");
        $func!(s05n3p02, "pngsuite");
        $func!(s06i3p02, "pngsuite");
        $func!(s06n3p02, "pngsuite");
        $func!(s07i3p02, "pngsuite");
        $func!(s07n3p02, "pngsuite");
        $func!(s08i3p02, "pngsuite");
        $func!(s08n3p02, "pngsuite");
        $func!(s09i3p02, "pngsuite");
        $func!(s09n3p02, "pngsuite");
        $func!(s32i3p04, "pngsuite");
        $func!(s32n3p04, "pngsuite");
        $func!(s33i3p04, "pngsuite");
        $func!(s33n3p04, "pngsuite");
        $func!(s34i3p04, "pngsuite");
        $func!(s34n3p04, "pngsuite");
        $func!(s35i3p04, "pngsuite");
        $func!(s35n3p04, "pngsuite");
        $func!(s36i3p04, "pngsuite");
        $func!(s36n3p04, "pngsuite");
        $func!(s37i3p04, "pngsuite");
        $func!(s37n3p04, "pngsuite");
        $func!(s38i3p04, "pngsuite");
        $func!(s38n3p04, "pngsuite");
        $func!(s39i3p04, "pngsuite");
        $func!(s39n3p04, "pngsuite");
        $func!(s40i3p04, "pngsuite");
        $func!(s40n3p04, "pngsuite");
        $func!(tbbn0g04, "pngsuite");
        $func!(tbbn2c16, "pngsuite");
        $func!(tbbn3p08, "pngsuite");
        $func!(tbgn2c16, "pngsuite");
        $func!(tbgn3p08, "pngsuite");
        $func!(tbrn2c08, "pngsuite");
        $func!(tbwn0g16, "pngsuite");
        $func!(tbwn3p08, "pngsuite");
        $func!(tbyn3p08, "pngsuite");
        $func!(tm3n3p02, "pngsuite");
        $func!(tp0n0g08, "pngsuite");
        $func!(tp0n2c08, "pngsuite");
        $func!(tp0n3p08, "pngsuite");
        $func!(tp1n3p08, "pngsuite");
        // Intentionally corrupted (not really useful for bench)
        // $func!(xc1n0g08, "pngsuite");
        // $func!(xc9n2c08, "pngsuite");
        // $func!(xcrn0g04, "pngsuite");
        // $func!(xcsn0g01, "pngsuite");
        // $func!(xd0n2c08, "pngsuite");
        // $func!(xd3n2c08, "pngsuite");
        // $func!(xd9n2c08, "pngsuite");
        // $func!(xdtn0g01, "pngsuite");
        // $func!(xhdn0g08, "pngsuite");
        // $func!(xlfn0g04, "pngsuite");
        // $func!(xs1n0g01, "pngsuite");
        // $func!(xs2n0g01, "pngsuite");
        // $func!(xs4n0g01, "pngsuite");
        // $func!(xs7n0g01, "pngsuite");
        $func!(z00n2c08, "pngsuite");
        $func!(z03n2c08, "pngsuite");
        $func!(z06n2c08, "pngsuite");
        $func!(z09n2c08, "pngsuite");
    };
}

#[cfg(feature = "bench-open")]
mod open {
    use super::{Bencher, Path};
    macro_rules! open_image {
        ($name:ident, $path:expr) => {
            #[bench]
            fn $name(b: &mut Bencher) {
                b.iter(|| {
                    rpng::Png::open(
                        Path::new("tests\\test_images")
                            .join(&format!("{}", $path))
                            .join(&format!("{}.png", stringify!($name))),
                    )
                    .unwrap()
                });
            }
        };
    }
    pngsuite!(open_image);
}

#[cfg(feature = "bench-pixels")]
mod pixels {
    use super::{Bencher, Path};
    macro_rules! open_image {
        ($name:ident, $path:expr) => {
            #[bench]
            fn $name(b: &mut Bencher) {
                let png = rpng::Png::open(
                    Path::new("tests\\test_images")
                        .join(&format!("{}", $path))
                        .join(&format!("{}.png", stringify!($name))),
                )
                .unwrap();
                b.iter(|| png.pixels());
            }
        };
    }
    pngsuite!(open_image);
}
