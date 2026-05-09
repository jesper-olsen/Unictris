use crate::shape::Shape;
use rand::prelude::*;

const LEVEL_TICK_INCREASE: u64 = 6000;
const FRAMES_PER_DROP: u64 = 30;
pub const BOARD_WIDTH: u8 = 10;
pub const BOARD_HEIGHT: u8 = 20;

pub struct Tetromino {
    pub x: u8, // shape location on the board (upper left)
    pub y: u8,
    pub orientation: u8, // 4 orientations: rotated 0, 90, 180 or 270 degrees
    pub shape: Shape,
}

impl Tetromino {
    pub fn new(rng: &mut ThreadRng) -> Self {
        let orientation = rng.random_range(0..4);
        let shape = Shape::random(rng);
        let (width, _) = shape.dim(orientation);
        let x = rng.random_range(0..BOARD_WIDTH - width + 1);
        Tetromino {
            shape,
            orientation,
            x,
            y: 0,
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
        for i in (0..row as usize).rev() {
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
    rng: ThreadRng,
}

impl Default for Game {
    fn default() -> Game {
        let mut rng = rand::rng();
        Game {
            tetromino: Tetromino::new(&mut rng),
            tick: 0,
            score: 0,
            board: Board::default(),
            paused: false,
            rng,
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
            let idx_x = x + self.tetromino.x;
            let idx_y = y + self.tetromino.y;
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

    fn wipe_filled_rows(&mut self) {
        let (_, height) = self.tetromino.shape.dim(self.tetromino.orientation);

        let start = self.tetromino.y;
        let end = start + height;

        let mut row = end;

        while row > start {
            row -= 1;

            if self.board.is_filled(row) {
                self.board.wipe(row);
                self.score += 1;

                // stay on same row because rows shifted down
                row += 1;
            }
        }
    }
    pub fn drop(&mut self) {
        while self.try_move(Move::Down) {}
        self.wipe_filled_rows()
    }

    pub fn left(&mut self) {
        self.try_move(Move::Left);
    }

    pub fn right(&mut self) {
        self.try_move(Move::Right);
    }

    pub fn rotate(&mut self) {
        self.try_move(Move::Rotate);
    }

    fn move_to_pos(&self, m: Move) -> Option<(u8, u8, u8)> {
        let (x, y, r) = (
            self.tetromino.x,
            self.tetromino.y,
            self.tetromino.orientation,
        );

        match m {
            Move::Left if x > 0 => Some((x - 1, y, r)),
            Move::Left => None,
            Move::Right => {
                let (width, _) = self.tetromino.shape.dim(r);
                if x + width < BOARD_WIDTH {
                    Some((x + 1, y, r))
                } else {
                    None
                }
            }
            Move::Down => {
                let (_, height) = self.tetromino.shape.dim(r);
                if y + 1 + height > BOARD_HEIGHT {
                    None
                } else {
                    Some((x, y + 1, r))
                }
            }
            Move::Rotate => {
                let new_r = (r + 1) % 4;
                // wall kick - shift left to make it fit
                let (width, _) = self.tetromino.shape.dim(new_r);
                let new_x = if x + width > BOARD_WIDTH {
                    BOARD_WIDTH - width
                } else {
                    x
                };
                Some((new_x, y, new_r))
            }
        }
    }

    fn check_hit(&mut self, x: u8, y: u8, r: u8) -> bool {
        self.clear_tetromino();
        let hit = self.tetromino.shape.coor(r).into_iter().any(|(sx, sy)| {
            y + sy >= BOARD_HEIGHT || x + sx >= BOARD_WIDTH || self.board.get(x + sx, y + sy) != 0
        });
        self.set_tetromino();
        hit
    }

    fn try_move(&mut self, m: Move) -> bool {
        let Some((x, y, r)) = self.move_to_pos(m) else {
            return false;
        };

        if self.check_hit(x, y, r) {
            return false;
        }

        // all clear - move tetramino --------------
        self.clear_tetromino();
        self.tetromino.x = x;
        self.tetromino.y = y;
        self.tetromino.orientation = r;
        self.set_tetromino();
        true
    }

    pub fn do_tick(&mut self) -> bool {
        if self.paused {
            return true;
        }
        self.tick = self.tick.wrapping_add(1);
        if self.tick % FRAMES_PER_DROP <= self.tick / LEVEL_TICK_INCREASE {
            // only update some of the time...
            if !self.try_move(Move::Down) {
                if self.tetromino.y == 0 {
                    return false; // overflow - game over
                }
                self.wipe_filled_rows();
                self.tetromino = Tetromino::new(&mut self.rng);
            }
        }
        true
    }
}
