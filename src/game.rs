use crate::shape::Shape;
use rand::prelude::*;

const LEVEL_TICK_INCREASE: u64 = 6000;
const FRAMES_PER_DROP: u64 = 30;
pub const BOARD_WIDTH: u8 = 10;
pub const BOARD_HEIGHT: u8 = 20;

pub struct Tetromino {
    pub px: u8, // shape location
    pub py: u8,
    pub orientation: u8,
    pub shape: Shape,
}

impl Tetromino {
    pub fn new() -> Self {
        let orientation = random::<u8>() % 4;
        let shape = Shape::new(random::<u8>() % 7);
        let (width, _) = shape.dim(orientation);
        Tetromino {
            shape,
            orientation,
            px: random::<u8>() % (BOARD_WIDTH - width),
            py: 0,
        }
    }
}

pub struct Board {
    board: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
}

impl Default for Board {
    fn default() -> Board {
        Board {
            board: [[0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
        }
    }
}

impl Board {
    fn set(&mut self, x: u8, y: u8, v: u8) {
        self.board[y as usize][x as usize] = v
    }

    pub fn get(&self, x: u8, y: u8) -> u8 {
        self.board[y as usize][x as usize]
    }

    fn is_filled(&self, row: u8) -> bool {
        self.board[row as usize].iter().all(|&v| v != 0)
    }

    fn wipe(&mut self, row: u8) {
        for i in (0..row).rev() {
            let i = i as usize;
            self.board[i + 1] = self.board[i];
        }
        self.board[0].fill(0);
    }
}

pub struct Game {
    pub tetromino: Tetromino, // active tetromino
    tick: u64,
    pub score: u32,
    pub board: Board,
    pub paused: bool,
}

impl Default for Game {
    fn default() -> Game {
        Game {
            tetromino: Tetromino::new(),
            tick: 0,
            score: 0,
            board: Board::default(),
            paused: false,
        }
    }
}

pub enum Move {
    Left,
    Right,
    Down,
    Rotate,
}

impl Game {
    fn draw_tetromino(&mut self, v: u8) {
        for (x, y) in self.tetromino.shape.coor(self.tetromino.orientation) {
            let idx_x = x + self.tetromino.px;
            let idx_y = y + self.tetromino.py;
            self.board.set(idx_x, idx_y, v);
        }
    }

    fn set_tetromino(&mut self) {
        self.draw_tetromino(self.tetromino.shape.kind() + 1);
    }

    fn clear_tetromino(&mut self) {
        self.draw_tetromino(0);
    }

    pub fn level(&self) -> u64 {
        1 + self.tick / LEVEL_TICK_INCREASE
    }

    pub fn wipe_filled_rows(&mut self) {
        let (_, height) = self.tetromino.shape.dim(self.tetromino.orientation);
        for row in self.tetromino.py..self.tetromino.py + height {
            if self.board.is_filled(row) {
                self.board.wipe(row);
                self.score += 1;
            }
        }
    }

    // move tetromino if it does not hit anything
    pub fn try_move(&mut self, m: Move) -> bool {
        let tet = &mut self.tetromino;
        let (x, y, r) = match m {
            Move::Left if tet.px > 0 => (tet.px - 1, tet.py, tet.orientation),
            Move::Right => {
                let (width, _) = tet.shape.dim(tet.orientation);
                if tet.px + width < BOARD_WIDTH {
                    (tet.px + 1, tet.py, tet.orientation)
                } else {
                    return false;
                }
            }
            Move::Down => (tet.px, tet.py + 1, tet.orientation),
            Move::Rotate => {
                let new_r = (tet.orientation + 1) % 4;
                // wall kick - shift left to make it fit
                let (width, _) = tet.shape.dim(new_r);
                let new_x = if tet.px + width > BOARD_WIDTH {
                    BOARD_WIDTH - width
                } else {
                    tet.px
                };
                (new_x, tet.py, new_r)
            }
            _ => return false,
        };

        let (_, height) = tet.shape.dim(r);
        if y + height > BOARD_HEIGHT {
            return false;
        }
        self.clear_tetromino();
        let hit = self.tetromino.shape.coor(r).into_iter().any(|(sx, sy)| {
            y + sy >= BOARD_HEIGHT || x + sx >= BOARD_WIDTH || self.board.get(x + sx, y + sy) != 0
        });
        self.set_tetromino();
        if !hit {
            self.clear_tetromino();
            (
                self.tetromino.px,
                self.tetromino.py,
                self.tetromino.orientation,
            ) = (x, y, r);
            self.set_tetromino();
        }
        !hit
    }

    pub fn do_tick(&mut self) -> bool {
        if self.paused {
            return true;
        }
        self.tick = (self.tick + 1) % u64::MAX;
        if self.tick % FRAMES_PER_DROP <= self.tick / LEVEL_TICK_INCREASE {
            // only update some of the time...
            if !self.try_move(Move::Down) {
                if self.tetromino.py == 0 {
                    return false; // overflow - game over
                }
                self.wipe_filled_rows();
                self.tetromino = Tetromino::new();
            }
        }
        true
    }
}
