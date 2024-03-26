pub fn up(prev: &[u8], raw_row: &[u8], decoded_row: &mut [u8]) {
    if prev.is_empty() {
        decoded_row[..].copy_from_slice(raw_row);
        return;
    }

    for i in 0..decoded_row.len() {
        let prev = prev[i];

        decoded_row[i] = raw_row[i].wrapping_add(prev)
    }
}

pub fn sub(raw_row: &[u8], decoded_row: &mut [u8], bytes_per_pixel: usize) {
    decoded_row[..bytes_per_pixel].copy_from_slice(&raw_row[..bytes_per_pixel]);

    for i in bytes_per_pixel..decoded_row.len() {
        let prev = decoded_row[i - bytes_per_pixel];

        decoded_row[i] = raw_row[i].wrapping_add(prev)
    }
}

pub fn average(prev: &[u8], raw_row: &[u8], decoded_row: &mut [u8], bytes_per_pixel: usize) {
    for i in 0..bytes_per_pixel {
        decoded_row[i] = raw_row[i].wrapping_add(prev[i] / 2);
    }

    for i in bytes_per_pixel..decoded_row.len() {
        let up = prev[i];
        let left = decoded_row[i - bytes_per_pixel];

        let val2 = ((u16::from(up) + u16::from(left)) / 2) as u8;
        let val = (up >> 1).wrapping_add(left >> 1) + (up & left & 0b1);

        if val2 != val {
            println!("{up}:{left}:{val2}:{val}");
        }

        decoded_row[i] = raw_row[i].wrapping_add(val);
    }
}

pub fn paeth(prev: &[u8], raw_row: &[u8], decoded_row: &mut [u8], bytes_per_pixel: usize) {
    for i in 0..bytes_per_pixel {
        let up = prev[i];
        let left = 0;
        let upper_left = 0;

        let val = paeth_predictor(left, i16::from(up), upper_left);

        decoded_row[i] = raw_row[i].wrapping_add(val)
    }

    for i in bytes_per_pixel..decoded_row.len() {
        let up = prev[i];
        let left = decoded_row[i - bytes_per_pixel];
        let upper_left = prev[i - bytes_per_pixel];

        let val = paeth_predictor(i16::from(left), i16::from(up), i16::from(upper_left));

        decoded_row[i] = raw_row[i].wrapping_add(val)
    }
}

// a = left, b = above, c = upper left
fn paeth_predictor(a: i16, b: i16, c: i16) -> u8 {
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();

    if pa <= pb && pa <= pc {
        (a % 256) as u8
    } else if pb <= pc {
        (b % 256) as u8
    } else {
        (c % 256) as u8
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
