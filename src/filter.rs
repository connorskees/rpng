use std::cmp::min;

pub fn sub(this_row: &[u8], chunk_size: u8, reverse: bool) -> Vec<Vec<u8>> {
    let mut chunks: Vec<Vec<u8>> = this_row
        .chunks(chunk_size as usize)
        .map(Vec::from)
        .collect();

    for pixel_idx in 1..chunks.len() {
        // start at 1 because first pixel is unchanged
        for rgba_idx in 0..chunks[pixel_idx].len() {
            let a = chunks[pixel_idx - 1][rgba_idx];
            if reverse {
                chunks[pixel_idx][rgba_idx] = chunks[pixel_idx][rgba_idx].wrapping_add(a);
            } else {
                chunks[pixel_idx][rgba_idx] = chunks[pixel_idx][rgba_idx].wrapping_sub(a);
            }
        }
    }

    chunks
}

pub fn up(
    this_row: &[u8],
    row_above: Option<&Vec<Vec<u8>>>,
    chunk_size: u8,
    reverse: bool,
) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row
        .chunks(chunk_size as usize)
        .map(Vec::from)
        .collect();
    let row_above: &Vec<Vec<u8>> = if let Some(ra) = row_above {
        ra
    } else {
        return this_row_chunks;
    };
    for pixel_idx in 0..this_row_chunks.len() {
        for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
            let b = row_above[pixel_idx][rgba_idx];
            if reverse {
                this_row_chunks[pixel_idx][rgba_idx] =
                    this_row_chunks[pixel_idx][rgba_idx].wrapping_add(b);
            } else {
                this_row_chunks[pixel_idx][rgba_idx] =
                    this_row_chunks[pixel_idx][rgba_idx].wrapping_sub(b);
            }
        }
    }
    this_row_chunks
}

pub fn average(this_row: &[u8], row_above: Option<&Vec<Vec<u8>>>, chunk_size: u8) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row
        .chunks(chunk_size as usize)
        .map(Vec::from)
        .collect();
    for pixel_idx in 0..this_row_chunks.len() {
        for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
            let a = if pixel_idx == 0 {
                0
            } else {
                this_row_chunks[pixel_idx - 1][rgba_idx]
            };
            let b: u8 = if let Some(val) = row_above {
                val[pixel_idx][rgba_idx]
            } else {
                0
            };
            this_row_chunks[pixel_idx][rgba_idx] = this_row_chunks[pixel_idx][rgba_idx]
                .wrapping_add(((u16::from(a) + u16::from(b)) / 2) as u8);
        }
    }
    this_row_chunks
}

pub fn paeth(
    this_row: &[u8],
    row_above: Option<&Vec<Vec<u8>>>,
    chunk_size: u8,
    reverse: bool,
) -> Vec<Vec<u8>> {
    let mut this_row_chunks: Vec<Vec<u8>> = this_row
        .chunks(chunk_size as usize)
        .map(Vec::from)
        .collect();
    let is_first_row: bool = row_above.is_none();
    let placeholder: &Vec<Vec<u8>> = &Vec::new();
    let above: &Vec<Vec<u8>> = if let Some(val) = row_above {
        val
    } else {
        placeholder
    };
    for pixel_idx in 0..this_row_chunks.len() {
        for rgba_idx in 0..this_row_chunks[pixel_idx].len() {
            let p: u8 = if pixel_idx == 0 {
                // the first pixel has no neighbors to the left, so we treat `a` and `c` as 0
                // paeth_predictor(0, b, 0) = b, so we can just directly set `p = b`
                if is_first_row {
                    0
                } else {
                    above[pixel_idx][rgba_idx]
                } // above
            } else {
                let a = this_row_chunks[pixel_idx - 1][rgba_idx]; // left
                let b = if is_first_row {
                    0
                } else {
                    above[pixel_idx][rgba_idx]
                }; // above
                let c = if is_first_row {
                    0
                } else {
                    above[pixel_idx - 1][rgba_idx]
                }; // above left
                paeth_predictor(i16::from(a), i16::from(b), i16::from(c))
            };
            if reverse {
                this_row_chunks[pixel_idx][rgba_idx] =
                    this_row_chunks[pixel_idx][rgba_idx].wrapping_add(p);
            } else {
                this_row_chunks[pixel_idx][rgba_idx] =
                    this_row_chunks[pixel_idx][rgba_idx].wrapping_sub(p);
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

    match min(min(pa, pb), pc) {
        // order here for ties is important
        diff if diff == pa => a as u8,
        diff if diff == pb => b as u8,
        diff if diff == pc => c as u8,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paeth_predictor() {
        assert_eq!(paeth_predictor(37, 84, 1), 84);
        assert_eq!(paeth_predictor(118, 128, 125), 118);
        assert_eq!(paeth_predictor(37, 84, 61), 61);
    }
}
