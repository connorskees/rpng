#[derive(Debug)]
pub enum Interlacing {
    None = 0,
    Adam7 = 1,
}

impl std::default::Default for Interlacing {
    fn default() -> Self {
        Self::None
    }
}

impl Interlacing {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => unimplemented!("adam7 interlacing is not currently supported"),//Self::Adam7,
            _ => panic!("Unknown value: {}", value),
        }
    }

    pub fn adam7(width: u32, height: u32, pixels: Vec<u8>) {
        /*
            variables declared and initialized elsewhere in the code:
                height, width
            functions or macros defined elsewhere in the code:
                visit(), min()
        */

        for pass in 1..=7 {
            
        }

        // int starting_row[7]  = { 0, 0, 4, 0, 2, 0, 1 };
        // int starting_col[7]  = { 0, 4, 0, 2, 0, 1, 0 };
        // int row_increment[7] = { 8, 8, 8, 4, 4, 2, 2 };
        // int col_increment[7] = { 8, 8, 4, 4, 2, 2, 1 };
        // int block_height[7]  = { 8, 8, 4, 4, 2, 2, 1 };
        // int block_width[7]   = { 8, 4, 4, 2, 2, 1, 1 };

        // long row, col;
        
        let mut pass = 0;
        while pass < 7 {
            // let row = starting_row[pass];
            // while (row < height) {
            //     col = starting_col[pass];
            //     while (col < width) {
            //         visit(row, col,
            //             std::cmp::min!(block_height[pass], height - row),
            //             std::cmp::min!(block_width[pass], width - col));
            //         col += col_increment[pass];
            //     }
            //     row += row_increment[pass];
            // }
            pass += 1;
        }

    }
}
