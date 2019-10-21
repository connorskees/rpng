use std::cmp::{min};


/**
 * IDEAS: 
 * - Use 8x8 image (duh)
 * 
 */

use crate::errors::MetadataError;
// use crate::filter::{/*undo_filter, */FilterType};
// use crate::common::Bitmap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
// TODO: rename to InterlaceMethod or similar
pub enum Interlacing {
    None = 0,
    Adam7 = 1,
}

impl std::default::Default for Interlacing {
    fn default() -> Self {
        Self::None
    }
}

pub fn visit(row: i64, column: i64, height: i64, width: i64, pass: usize, _ogwidth: u32, _ogheight: u32) -> Vec<Vec<u8>> {
    if pass == 6 {
        println!("r: {} c: {} - w: {} - h: {} - pass: {}", row, column, width, height, pass);
    }
    let raw = [0u8, 255, 0, 0, 255, 0, 0, 0, 255, 2, 1, 0, 255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 255, 2, 0, 255, 0, 0, 1, 0, 0, 255, 255, 0, 255, 1, 0, 0, 0, 0, 0, 0, 255, 0, 0, 255, 0, 0, 0, 255, 2, 1, 0, 255, 0, 0, 255, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255, 0, 0, 255, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 255, 255, 0, 0, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255, 0, 0, 255, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 255, 255, 0, 0, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let chunked: Vec<Vec<u8>> = raw.chunks_exact(37).map(Vec::from).collect();
    let mut scanline: Vec<Vec<u8>> = Vec::new();
    if pass == 6 {
        scanline.push(chunked[row as usize][column as usize..(4+column) as usize].to_vec());
    }
    // if (row == 0 && column != 0) || (row != 0 && column == 0) {
    //     let val = (max(row, 1)*max(column, 1)) as usize;
    //     scanline.extend(raw[val..=(4+val)].to_vec());
    // } else {
    //     scanline.extend(raw[0..=4].to_vec());
    // }
    // println!("{:?}", row);

    // for r in row..(row+height) {
    //     for c in column..(column+width) {
    //         scanline.push(chunked[r as usize][c as usize]);
    //         if (r == 0 && c != 0) || (r != 0 && c == 0) {
    //             scanline.push(raw[(max(r, 1)*max(c, 1)) as usize]);
    //         } else {
    //             scanline.push(raw[0]);
    //         }
    //     }
    // }
    scanline
}

impl Interlacing {
    pub fn from_u8(interlacing_type: u8) -> Result<Self, MetadataError> {
        match interlacing_type {
            0 => Ok(Self::None),
            1 => Ok(Self::Adam7),
            _ => Err(MetadataError::UnrecognizedInterlacingType{ interlacing_type }),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Adam7 => 1,
        }
    }

    pub fn adam7(width: u32, height: u32, _raw: Vec<u8>) -> Vec<Vec<Vec<u8>>> {
        /*
            variables declared and initialized elsewhere in the code:
                height, width
            functions or macros defined elsewhere in the code:
                visit(), min()
        */
        let mut deconstructed: Vec<Vec<Vec<u8>>> = Vec::new();

        let starting_row  = [ 0u8, 0, 4, 0, 2, 0, 1 ];
        let starting_col  = [ 0u8, 4, 0, 2, 0, 1, 0 ];
        let row_increment = [ 8u8, 8, 8, 4, 4, 2, 2 ];
        let col_increment = [ 8u8, 8, 4, 4, 2, 2, 1 ];
        let block_height  = [ 8u8, 8, 4, 4, 2, 2, 1 ];
        let block_width   = [ 8u8, 4, 4, 2, 2, 1, 1 ];

        // let mut count = 0;
        let mut row: i64;
        let mut col: i64;
        
        for pass in 0..=6  {
            row = i64::from(starting_row[pass]);
            while row < i64::from(height) {
                col = i64::from(starting_col[pass]);
                while col < i64::from(width) {
                    let height = min(i64::from(block_height[pass]), i64::from(height) - row);
                    let width = min(i64::from(block_width[pass]), i64::from(width) - col);
                    for r in row..(row+height) {
                        for c in col..(col+width) {
                            
                        }
                    }
                    // deconstructed.push(visit(row, col,
                    //     ,
                    //     ,
                    //     pass,
                    //     width, height
                    // ));
                    // count += 1;
                    col += i64::from(col_increment[pass]);
                }
                row += i64::from(row_increment[pass]);
            }
        }
        // println!("{:?}", &deconstructed[0..50]);
        // deconstructed
        unimplemented!()
    }
}


//   static adam7 (data, width, height, png) {
//     let bpp = png.colors
//     let startingRow = [0, 0, 4, 0, 2, 0, 1]
//     let startingCol = [0, 4, 0, 2, 0, 1, 0]
//     let rowIncrement = [8, 8, 8, 4, 4, 2, 2]
//     let colIncrement = [8, 8, 4, 4, 2, 2, 1]
//     let pass
//     let row, col
//     // let colorData = Buffer.alloc(width * height * 4)
//     let colorData = new Uint8Array(width * height * 4)
//     let inputOffset = 0
//     let prevInputOffset = 0
//     let heightIndex = 0
//     let scanLine = null
//     let prevScanLine = null
//     let filterType = null
//     let scanLineData
//     let scanLineDataList = []

//     pass = 0
//     while (pass < 7) {
//       heightIndex = 0
//       prevScanLine = null
//       row = startingRow[pass]
//       while (row < height) {
//         col = startingCol[pass]
//         filterType = data[inputOffset]
//         inputOffset += 1
//         prevInputOffset = inputOffset
//         while (col < width) {
//           inputOffset += bpp
//           col = col + colIncrement[pass]
//         }
//         scanLine = data.slice(prevInputOffset, inputOffset)
//         scanLineData = reverseFilter[filterType]({
//           bpp: bpp,
//           heightIndex: heightIndex,
//           data: scanLine,
//           prevData: prevScanLine,
//           png: png
//         })
//         heightIndex++
//         prevScanLine = scanLineData
//         scanLineDataList.push(scanLineData)
//         row = row + rowIncrement[pass]
//       }
//       pass = pass + 1
//     }

//     // reInit
//     inputOffset = 0
//     heightIndex = 0
//     scanLine = null
//     prevScanLine = null
//     filterType = null
//     let scanLineBuffer = Buffer.concat(scanLineDataList)
//     pass = 0
//     while (pass < 7) {
//       row = startingRow[pass]
//       while (row < height) {
//         col = startingCol[pass]
//         heightIndex++
//         while (col < width) {
//           for (let i = 0; i < bpp; i++) {
//             colorData[row * width * 4 + col * 4 + i] = scanLineBuffer[inputOffset + i]
//           }
//           if (bpp < 4) {
//             colorData[row * width * 4 + col * 4 + 3] = 255
//           }
//           inputOffset += bpp
//           col = col + colIncrement[pass]
//         }
//         row = row + rowIncrement[pass]
//       }
//       pass = pass + 1
//     }
//     return colorData
//   }
// }


    // fn adam7_multiplier_offset(pass: u8) -> [u8; 4] {
    //     // match pass {
    //     //     0 => [3, 0, 3, 0],
    //     //     1 => [3, 4, 3, 0],
    //     //     2 => [2, 0, 3, 4],
    //     //     3 => [2, 2, 2, 0],
    //     //     4 => [1, 0, 2, 2],
    //     //     5 => [1, 1, 1, 0],
    //     //     6 => [0, 0, 1, 1],
    //     //     _ => unreachable!()
    //     // }
    //     [
    //       3 - (pass >> 1),
    //       if pass & 1 == 0 { 0 } else {8 >> ( (pass + 1) >> 1) },
    //       if pass == 0 { 3 } else { 3 - ((pass - 1) >> 1) },
    //       if pass == 0 || pass & 1 == 1 { 0 } else { 8 >> (pass >> 1) },
    //     ]
    // }

    // pub fn adam7_pass_size(pass: u8, original_width: u32, original_height: u32) -> [u32; 2] {
    //     let [x_shift, x_offset, y_shift, y_offset] = Interlacing::adam7_multiplier_offset(pass);
    //     [
    //       (original_width  - u32::from(x_offset + (1 << x_shift) - 1)) >> x_shift,
    //       (original_height - u32::from(y_offset + (1 << y_shift) - 1)) >> y_shift,
    //     ]
    // }

    // pub fn adam7_extract_pass(pass: u8, canvas: &Bitmap<u8>) -> Vec<&Vec<u8>> {
    //     let [x_shift, x_offset, y_shift, y_offset] = Interlacing::adam7_multiplier_offset(pass);
    //     let mut sm_pixels: Vec<&Vec<u8>> = Vec::new();

    //     let width = height = 9;

    //     for y in ((y_offset.into())..=(height - 1)).step_by(1 << y_shift) {
    //         for x in ((x_offset.into())..=(width - 1)).step_by(1 << x_shift) {
    //             sm_pixels.push(&canvas[[x, y]]);
    //         }
    //     }

    //     sm_pixels

    //     // println!("{:?}", sm_pixels);

    //     // new_canvas_args = adam7_pass_size(pass, canvas.width, canvas.height) + [sm_pixels]
    //     // ChunkyPNG::Canvas.new(*new_canvas_args)
    // }

    // pub fn adam7_merge_pass(pass: u8, mut canvas: Bitmap<u8>, subcanvas: &Bitmap<u8>) {
    //     let [x_shift, x_offset, y_shift, y_offset] = Interlacing::adam7_multiplier_offset(pass);
    //     for y in 0..subcanvas.height() {
    //         for x in 0..subcanvas.width() {
    //             let new_x: usize = (x << x_shift) | (x_offset as usize);
    //             let new_y: usize = (y << y_shift) | (y_offset as usize);
    //             let new_pix = &subcanvas[[x, y]];
    //             canvas.set_pixel(new_x, new_y, new_pix);//[[, ]] = ;
    //         }
    //     }
    // }