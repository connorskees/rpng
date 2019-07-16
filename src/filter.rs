use std::vec::Vec;
use std::cmp::min;
use crate::errors::FilterError;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterType {
    /// No filter is applied
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

impl FilterType {
    pub fn from_u8(val: u8) -> Result<Self, FilterError> {
        match val {
            0 => Ok(Self::None),
            1 => Ok(Self::Sub),
            2 => Ok(Self::Up),
            3 => Ok(Self::Average),
            4 => Ok(Self::Paeth),
            _ => Err(FilterError::UnrecognizedFilterType(val))
        }
    }
}

impl std::default::Default for FilterType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterMethod {
    /// The default filter method. This exposes 5 filter algorithms: none, sub, up, average, and paeth
    Adaptive = 0,
}

impl FilterMethod {
    pub fn from_u8(val: u8) -> Result<Self, FilterError> {
        match val {
            0 => Ok(Self::Adaptive),
            _ => Err(FilterError::UnrecognizedFilterMethod(val))
        }
    }
}

impl std::default::Default for FilterMethod {
    fn default() -> Self {
        Self::Adaptive
    }
}

pub fn sub(this_row: &[u8], chunk_size: u8, reverse: bool) -> Vec<Vec<u8>> {
    let mut chunks: Vec<Vec<u8>> = this_row.chunks(chunk_size as usize).map(Vec::from).collect();
    for idx1 in 1..chunks.len() { // start at 1 because first pixel (0th) is initial
        for idx2 in 0..chunks[idx1].len() {
            let a = chunks[idx1-1][idx2];
            if reverse {
                chunks[idx1][idx2] = chunks[idx1][idx2].wrapping_add(a);
            } else {
                chunks[idx1][idx2] = chunks[idx1][idx2].wrapping_sub(a);
            }
        }
    }
    chunks
}

pub fn up(this_row: &[u8], row_above: Option<&Vec<Vec<u8>>>, chunk_size: u8, reverse: bool) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row.chunks(chunk_size as usize).map(Vec::from).collect();
    let row_above: &Vec<Vec<u8>> = if let Some(ra) = row_above { ra } else { return this_row_chunks };
    for idx1 in 0..this_row_chunks.len() {
        for idx2 in 0..this_row_chunks[idx1].len() {
            let b = row_above[idx1][idx2];
            if reverse {
                this_row_chunks[idx1][idx2] = this_row_chunks[idx1][idx2].wrapping_add(b);
            } else {
                this_row_chunks[idx1][idx2] = this_row_chunks[idx1][idx2].wrapping_sub(b);
            }
        }
    }
    this_row_chunks
}

pub fn average(this_row: &[u8], row_above: Option<&Vec<Vec<u8>>>, chunk_size: u8) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row.chunks(chunk_size as usize).map(Vec::from).collect();
    for pixel_idx in 0..this_row_chunks.len() { // start at 1 because first pixel (0th) is initial
        for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
            let a = if pixel_idx == 0 { 0 } else { this_row_chunks[pixel_idx-1][rgba_idx] };
            let b: u8 = if let Some(val) = row_above { val[pixel_idx][rgba_idx] } else { 0 };
            this_row_chunks[pixel_idx][rgba_idx] = this_row_chunks[pixel_idx][rgba_idx].wrapping_add(((u16::from(a) + u16::from(b)) / 2) as u8);
        }
    }
    this_row_chunks
}

pub fn paeth(this_row: &[u8], row_above: Option<&Vec<Vec<u8>>>, chunk_size: u8, reverse: bool) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row.chunks(chunk_size as usize).map(Vec::from).collect();
    let is_first_row: bool = row_above == None;
    let placeholder: &Vec<Vec<u8>> = &Vec::new();
    let above: &Vec<Vec<u8>> = if let Some(val) = row_above { val } else { placeholder };
    for pixel_idx in 0..this_row_chunks.len() { // start at 1 because first pixel (0th) is initial
        for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
            let p: u8 = if pixel_idx == 0 {
                // the first pixel has no neighbors to the left, so we treat `a` and `c` as 0
                // paeth_predictor(0, b, 0) = b, so we can just directly set `p = b` 
                if is_first_row { 0 } else { above[pixel_idx][rgba_idx] } // above
            } else {
                let a = this_row_chunks[pixel_idx-1][rgba_idx]; // left
                let b = if is_first_row { 0 } else { above[pixel_idx][rgba_idx] }; // above
                let c = if is_first_row { 0 } else { above[pixel_idx-1][rgba_idx] }; // above left
                paeth_predictor(i16::from(a), i16::from(b), i16::from(c))
            };
            if reverse {
                this_row_chunks[pixel_idx][rgba_idx] = this_row_chunks[pixel_idx][rgba_idx].wrapping_add(p);
            } else {
                this_row_chunks[pixel_idx][rgba_idx] = this_row_chunks[pixel_idx][rgba_idx].wrapping_sub(p);
            }
        }
    }
    this_row_chunks
}


fn paeth_predictor(a: i16, b: i16, c: i16) -> u8 {
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();

    match min(min(pa, pb), pc) { // order here for ties is important
        diff if diff == pa => a as u8,
        diff if diff == pb => b as u8,
        diff if diff == pc => c as u8,
        _ => panic!("error in paeth predictor (this indicates an error in the library. please file an issue at https://github.com/ConnorSkees/rpng/issues)")
    }
}