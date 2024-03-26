//! # Library for working with PNG files

#![warn(missing_debug_implementations)]

#[cfg(feature = "serialize")]
use serde_json;
#[cfg(feature = "serialize")]
use std::fs::File;
#[cfg(feature = "serialize")]
use std::io::Write;

pub use crate::common::*;
pub use crate::decoder::PngDecoder;
pub use crate::filter::*;
pub use png::{Png, PngBuilder};

pub mod chunks;
mod common;
mod decoder;
mod encoder;
pub mod errors;
mod filter;
mod interlacing;
mod png;
