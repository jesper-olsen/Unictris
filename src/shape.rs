use std::fmt;

// Tetrominos - packed into 7 16-bit numbers.
// Each tetromino shape is 4 squares inside a 4x4 block - we store x and y coordinate for each square,
// hence we need need 4*(2+2)=16 bits to describe one shape,
static BLOCK: [u16; 7] = [0x2154, 0x6510, 0x5140, 0x9840, 0x1654, 0x3210, 0x8951];

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..4 {
            let (w, h) = self.dim(r);
            writeln!(f, "Shape {r} dim: {w}x{h}")?;
            let mut a =
                [[false; Shape::TETROMINO_WIDTH as usize]; Shape::TETROMINO_HEIGHT as usize];
            for (x, y) in self.coor(r) {
                a[y as usize][x as usize] = true;
            }
            for row in a {
                for b in row {
                    write!(f, "{} ", if b { " X " } else { " O " })?
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

pub struct Shape(u8);

impl Shape {
    const TETROMINO_HEIGHT: u8 = 4;
    const TETROMINO_WIDTH: u8 = 4;

    pub const fn new(kind: u8) -> Self {
        Shape(kind)
    }

    pub const fn kind(&self) -> u8 {
        self.0
    }

    // rotate (x,y) coordinates by 0, 90, 180 or 270 degrees
    const fn rotate(x: u8, y: u8, r: u8) -> (u8, u8) {
        match r {
            0 => (x, y),
            1 => (Shape::TETROMINO_HEIGHT - 1 - y, x),
            2 => (
                Shape::TETROMINO_WIDTH - 1 - x,
                Shape::TETROMINO_HEIGHT - 1 - y,
            ),
            3 => (y, Shape::TETROMINO_WIDTH - 1 - x),
            _ => unimplemented!(),
        }
    }

    // each shape has 4 blocks on - return x,y of those four blocks
    pub const fn coor(&self, r: u8) -> [(u8, u8); 4] {
        let mut a = [(0, 0); 4];
        let mut min_x = u8::MAX;
        let mut min_y = u8::MAX;
        let mut i = 0;
        while i < Shape::TETROMINO_WIDTH as usize {
            let block = BLOCK[self.0 as usize];
            let x = (3 & block >> 4 * i + 2) as u8;
            let y = (3 & block >> 4 * i) as u8;
            a[i] = Self::rotate(x, y, r);
            min_x = if min_x <= a[i].0 { min_x } else { a[i].0 };
            min_y = if min_y <= a[i].1 { min_y } else { a[i].1 };
            i += 1;
        }
        i = 0;
        while i < Shape::TETROMINO_WIDTH as usize {
            a[i].0 -= min_x;
            a[i].1 -= min_y;
            i += 1;
        }
        a
    }

    // width, height of shape
    pub const fn dim(&self, r: u8) -> (u8, u8) {
        let mut max_x = u8::MIN;
        let mut max_y = u8::MIN;
        let a = self.coor(r);
        let mut i = 0;
        while i < a.len() {
            max_x = if max_x >= a[i].0 { max_x } else { a[i].0 };
            max_y = if max_y >= a[i].1 { max_y } else { a[i].1 };
            i += 1;
        }
        (max_x + 1, max_y + 1)
    }
}
