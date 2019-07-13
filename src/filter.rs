use std::vec::Vec;
use std::cmp::min;

pub fn sub(this_row: &[u8], chunk_size: u8, reverse: bool) -> Vec<Vec<u8>> {
    let mut chunks: Vec<Vec<u8>> = this_row.chunks(chunk_size as usize).map(|x| Vec::from(x)).collect();
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

pub fn up(this_row: &[u8], row_above: &Vec<Vec<u8>>, chunk_size: u8, reverse: bool) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row.chunks(chunk_size as usize).map(|x| Vec::from(x)).collect();
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

pub fn average(this_row: &[u8], row_above: &Vec<Vec<u8>>, chunk_size: u8) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row.chunks(chunk_size as usize).map(|x| Vec::from(x)).collect();
    for pixel_idx in 1..this_row_chunks.len() { // start at 1 because first pixel (0th) is initial
        for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
            let a = this_row_chunks[pixel_idx-1][rgba_idx];
            let b = row_above[pixel_idx][rgba_idx];
            this_row_chunks[pixel_idx][rgba_idx] = this_row_chunks[pixel_idx][rgba_idx].wrapping_add((a + b) / 2);
        }
    }
    this_row_chunks
}

pub fn paeth(this_row: &[u8], row_above: &Vec<Vec<u8>>, chunk_size: u8, reverse: bool) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row.chunks(chunk_size as usize).map(|x| Vec::from(x)).collect();
    for pixel_idx in 0..this_row_chunks.len() { // start at 1 because first pixel (0th) is initial
        for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
            let p: u8;
            if pixel_idx == 0 {
                // the first pixel has no neighbors to the left, so we treat `a` and `c` as 0
                // paeth_predictor(0, b, 0) = b, so we can just directly set `p = b` 
                p = row_above[pixel_idx][rgba_idx]; // above
            } else {
                let a = this_row_chunks[pixel_idx-1][rgba_idx]; // left
                let b = row_above[pixel_idx][rgba_idx]; // above
                let c = row_above[pixel_idx-1][rgba_idx]; // above left
                p = paeth_predictor(a as i16, b as i16, c as i16);
            }
            if reverse {
                this_row_chunks[pixel_idx][rgba_idx] = this_row_chunks[pixel_idx][rgba_idx].wrapping_add(p);
            }
            this_row_chunks[pixel_idx][rgba_idx] = this_row_chunks[pixel_idx][rgba_idx].wrapping_sub(p);
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
        x if x == pa => a as u8,
        x if x == pb => b as u8,
        x if x == pc => c as u8,
        _ => panic!("err")
    }
}