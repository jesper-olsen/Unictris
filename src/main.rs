use crossterm::{
    QueueableCommand, cursor,
    event::{Event, KeyCode, KeyEvent, poll, read},
    style::{self, Stylize},
    terminal,
};
use rand::prelude::*;
use std::io::{Result, Write, stdout};
use std::time;

mod shape;

use shape::Shape;

const LEVEL_TICK_INCREASE: u64 = 6000;
const FRAMES_PER_DROP: u64 = 30;
const BOARD_WIDTH: u8 = 10;
const BOARD_HEIGHT: u8 = 20;

struct Game {
    px: u8, // shape location
    py: u8,
    orientation: u8,
    shape: Shape,
    tick: u64,
    score: u32,
    board: [[u8; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    paused: bool,
}

fn centered_x(s: &str) -> u16 {
    let leftedge: u16 = 25;
    let n: u16 = s.len().try_into().unwrap();

    match terminal::size() {
        Ok((cols, _rows)) => {
            if cols < leftedge + n {
                leftedge
            } else {
                (cols - leftedge - n) / 2 + leftedge
            }
        }
        Err(_) => leftedge,
    }
}

fn render_game_info(g: &Game) -> Result<()> {
    let s1: &str = "Unictris - Unicode-powered Tetris";
    let s2 = "Rusty Glyph Edition 2025 ";

    crossterm::queue!(
        stdout(),
        cursor::MoveTo(centered_x(s1), 2),
        style::PrintStyledContent(s1.cyan()),
        cursor::MoveTo(centered_x(s2), 3),
        style::PrintStyledContent(s2.yellow()),
    )?;

    let i = centered_x("Score : 123456"); /* get a pos base on av score digits */
    crossterm::queue!(
        stdout(),
        cursor::MoveTo(i, 5.try_into().unwrap()),
        style::PrintStyledContent(format!("Score : {}", g.score).bold().white()),
        cursor::MoveTo(i, 6.try_into().unwrap()),
        style::PrintStyledContent(format!("Level : {}", g.level()).bold().white()),
        cursor::MoveTo(i, 8.try_into().unwrap()),
        style::PrintStyledContent(
            format!("Shape : {}.{}", g.shape.kind(), g.orientation)
                .bold()
                .white()
        ),
    )?;
    Ok(())
}

fn draw_screen(g: &Game) -> Result<()> {
    let mut stdout = stdout();

    for (i, row) in g.board.iter().enumerate() {
        let i: u16 = (i.try_into()).expect("board too big");

        crossterm::queue!(stdout, cursor::MoveTo(1, i + 1))?;
        for (j, v) in row.iter().enumerate() {
            let v = *v as u32;
            let j: u16 = j.try_into().unwrap();
            crossterm::queue!(stdout, cursor::MoveTo(j * 2 + 1, i + 1)).ok();
            let s = match v {
                0 => "  ".white(),
                1 => "\u{16A0}\u{16A0}".on_red(),
                2 => "\u{16A2}\u{16A2}".on_red(),
                3 => "\u{16A5}\u{16A5}".on_red(),
                4 => "\u{16A6}\u{16A6}".on_red(),
                5 => "\u{16BC}\u{16BC}".on_red(),
                6 => "\u{16AD}\u{16AD}".on_red(),
                _ => "\u{16D2}\u{16D2}".on_red(),
                // 1 => "●●".on_blue(),
                // 2 => "◎◎".blue().on_yellow(),
                // 3 => "□□".on_green(),
                // 4 => "◦◦".on_magenta(),
                // 5 => "○○".on_dark_red(),
                // 6 => "◼◼".on_cyan(),
                // _ => "◉◉".on_red(),
                // 1 => "  ".on_blue(),
                // 2 => "  ".on_yellow(),
                // 3 => "  ".on_green(),
                // 4 => "  ".on_magenta(),
                // 5 => "  ".on_dark_red(),
                // 6 => "  ".on_cyan(),
                // _ => "  ".on_red(),
            };
            crossterm::queue!(
                stdout,
                style::PrintStyledContent(s),
                cursor::MoveTo((j + 1) * 2 + 1, i + 1)
            )?
        }
    }
    render_game_info(g)?;
    stdout.flush()
}

enum Move {
    Left,
    Right,
    Down,
    Rotate,
}

impl Game {
    fn draw_tetromino(&mut self, v: u8) {
        for (x, y) in self.shape.coor(self.orientation) {
            let idx_x = x + self.px;
            let idx_y = y + self.py;
            self.board[idx_y as usize][idx_x as usize] = v;
        }
    }

    fn set_tetromino(&mut self) {
        self.draw_tetromino(self.shape.kind() + 1);
    }

    fn clear_tetromino(&mut self) {
        self.draw_tetromino(0);
    }

    fn level(&self) -> u64 {
        1 + self.tick / LEVEL_TICK_INCREASE
    }

    fn new_tetromino(&mut self) {
        self.shape = Shape::new(random::<u8>() % 7);
        self.orientation = random::<u8>() % 4;
        let (width, _) = self.shape.dim(self.orientation);
        self.px = random::<u8>() % (BOARD_WIDTH - width);
        self.py = 0;
    }

    fn wipe_filled_rows(&mut self) {
        let (_, height) = self.shape.dim(self.orientation);
        for row in self.py..self.py + height {
            if self.board[row as usize].iter().all(|&v| v != 0) {
                for i in (1..row).rev() {
                    let i = i as usize;
                    self.board[i + 1] = self.board[i];
                }
                self.board[0].fill(0);
                self.score += 1;
            }
        }
    }

    // move tetromino if it does not hit anything
    fn try_move(&mut self, m: Move) -> bool {
        let (x, y, r) = match m {
            Move::Left if self.px > 0 => (self.px - 1, self.py, self.orientation),
            Move::Right => {
                let (width, _) = self.shape.dim(self.orientation);
                if self.px + width < BOARD_WIDTH {
                    (self.px + 1, self.py, self.orientation)
                } else {
                    return false;
                }
            }
            Move::Down => (self.px, self.py + 1, self.orientation),
            Move::Rotate => {
                let new_r = (self.orientation + 1) % 4;
                // wall kick - shift left to make it fit
                let (width, _) = self.shape.dim(new_r);
                let new_x = if self.px + width > BOARD_WIDTH {
                    BOARD_WIDTH - width
                } else {
                    self.px
                };
                (new_x, self.py, new_r)
            }
            _ => return false,
        };

        let (_, height) = self.shape.dim(r);
        if y + height > BOARD_HEIGHT {
            return false;
        }
        self.clear_tetromino();
        let hit = self.shape.coor(r).into_iter().any(|(sx, sy)| {
            y + sy >= BOARD_HEIGHT
                || x + sx >= BOARD_WIDTH
                || self.board[(y + sy) as usize][(x + sx) as usize] != 0
        });
        self.set_tetromino();
        if !hit {
            self.clear_tetromino();
            (self.px, self.py, self.orientation) = (x, y, r);
            self.set_tetromino();
        }
        !hit
    }

    fn do_tick(&mut self) -> bool {
        if self.paused {
            return true;
        }
        self.tick = (self.tick + 1) % u64::MAX;
        if self.tick % FRAMES_PER_DROP <= self.tick / LEVEL_TICK_INCREASE {
            // only update some of the time...
            if !self.try_move(Move::Down) {
                if self.py == 0 {
                    return false; // overflow - game over
                }
                self.wipe_filled_rows();
                self.new_tetromino();
            }
        }
        true
    }
}

fn runloop(g: &mut Game) -> Result<()> {
    while g.do_tick() {
        if let Ok(true) = poll(time::Duration::from_millis(10)) {
            match read() {
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                })) => return Ok(()),
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    ..
                })) => g.paused = !g.paused,
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                })) => {
                    g.try_move(Move::Left);
                }
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                })) => {
                    g.try_move(Move::Right);
                }
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                })) => {
                    while g.try_move(Move::Down) {
                        continue;
                    }
                    g.wipe_filled_rows();
                    g.new_tetromino();
                }
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                })) => {
                    g.try_move(Move::Rotate);
                }
                _ => (),
            }
        }
        draw_screen(g)?;
    }
    Ok(())
}

fn box_(x: u16, y: u16, width: u16, height: u16) -> Result<()> {
    const TOP_LEFT: &str = "\u{250f}";
    const TOP_RIGHT: &str = "\u{2513}";
    const BOTTOM_LEFT: &str = "\u{2517}";
    const BOTTOM_RIGHT: &str = "\u{251b}";
    const VERTICAL: &str = "\u{2503}";
    const HORIZONTAL: &str = "\u{2501}";
    let mut stdout = stdout();

    stdout
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::MoveTo(x, y))?
        .queue(style::PrintStyledContent(TOP_LEFT.white()))?
        .queue(cursor::MoveTo(x + width, y))?
        .queue(style::PrintStyledContent(TOP_RIGHT.white()))?
        .queue(cursor::MoveTo(x, y + height))?
        .queue(style::PrintStyledContent(BOTTOM_LEFT.white()))?
        .queue(cursor::MoveTo(x + width, y + height))?
        .queue(style::PrintStyledContent(BOTTOM_RIGHT.white()))?;

    for i in 1..width {
        crossterm::queue!(
            stdout,
            cursor::MoveTo(x + i, y),
            style::PrintStyledContent(HORIZONTAL.white()),
            cursor::MoveTo(x + i, y + height),
            style::PrintStyledContent(HORIZONTAL.white())
        )?;
    }
    for i in 1..height {
        crossterm::queue!(
            stdout,
            cursor::MoveTo(x, y + i),
            style::PrintStyledContent(VERTICAL.white()),
            cursor::MoveTo(x + width, y + i),
            style::PrintStyledContent(VERTICAL.white())
        )?;
    }
    crossterm::queue!(
        stdout,
        cursor::Hide,
        cursor::MoveTo(x + width + 2, y + height + 2)
    )?;

    stdout.flush()
}

fn main() -> Result<()> {
    let mut game = Game {
        px: 0,
        py: 0,
        orientation: 0,
        shape: Shape::new(0),
        tick: 0,
        score: 0,
        board: [[0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
        paused: false,
    };
    game.new_tetromino();

    crossterm::queue!(
        stdout(),
        style::ResetColor,
        terminal::Clear(terminal::ClearType::All),
        terminal::EnterAlternateScreen,
        cursor::Hide,
        cursor::MoveTo(0, 0)
    )?;
    terminal::enable_raw_mode()?;
    box_(0, 0, 21, 21)?;
    runloop(&mut game)?;

    crossterm::queue!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        terminal::LeaveAlternateScreen,
        cursor::Show,
        cursor::MoveTo(0, 0)
    )?;
    terminal::disable_raw_mode()?;

    println!("Score: {}; Level: {}", game.score, game.level());
    Ok(())
}
