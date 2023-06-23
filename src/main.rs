use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent},
    style::{self, Stylize},
    terminal, QueueableCommand, Result,
};
use rand::prelude::*;
use std::io::{stdout, Write};
use std::time;

// Tetrominos - packed into 7 64 bit numbers.
// Each tetromino is 4 squares - needs 4*(2+2)=16 bits to describe.
// Hence 448 bits in total: 7 tetrominos * 4 orientations * 16 bits.
static BLOCK: [u64; 7] = [
    0x2154_9540_2154_9540,
    0x6510_8451_6510_8451,
    0x5140_5140_5140_5140,
    0x9840_2140_9510_2654,
    0x1654_5840_5210_4951,
    0x3210_c840_3210_c840,
    0x8951_6540_1840_6210,
];

const TICK_LEVEL: u64 = 6000;

struct Game {
    x: u8, // coor
    y: u8,
    r: u8,  // orientation
    px: u8, // old coor
    py: u8,
    pr: u8,
    p: u8, // tetromino
    tick: u64,
    score: u32,
    board: [[u8; 10]; 20], // 20 rows x 10 cols
    paused: bool,
}

// extract a bit packed number from a block
fn num(p: u8, r: u8, i: u8) -> u8 {
    (3 & BLOCK[p as usize] >> (r * 16 + i)) as u8
}

// calculate width-1 for tetromino
fn width(p: u8, r: u8) -> u8 {
    let p = (0..4).map(|i| num(p, r, i * 4 + 2)).fold((0, 9), |m, v| {
        (std::cmp::max(m.0, v), std::cmp::min(m.1, v))
    });
    p.0 - p.1
}

// calculate height-1 for tetromino
fn height(p: u8, r: u8) -> u8 {
    let p = (0..4).map(|i| num(p, r, i * 4)).fold((0, 9), |m, v| {
        (std::cmp::max(m.0, v), std::cmp::min(m.1, v))
    });
    p.0 - p.1
}

fn new_tetramino(g: &mut Game) {
    g.p = random::<u8>() % 7; // tetromino
    g.r = random::<u8>() % 4; // orientation
    g.x = random::<u8>() % (10 - width(g.p, g.r));
    g.y = 0;
    g.py = 0;
    g.pr = g.r;
    g.px = g.x;
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

fn level(g: &Game) -> u64 {
    1 + g.tick / TICK_LEVEL
}

fn render_game_info(g: &Game) {
    let s1: &str = "Unictris - Unicode-powered Tetris";
    let s2 = "Rusty Glyph Edition 2023 ";

    crossterm::queue!(
        stdout(),
        cursor::MoveTo(centered_x(s1), 2),
        style::PrintStyledContent(s1.cyan()),
        cursor::MoveTo(centered_x(s2), 3),
        style::PrintStyledContent(s2.yellow()),
    )
    .ok();

    let i = centered_x("Score : 123456"); /* get a pos base on av score digits */
    crossterm::queue!(
        stdout(),
        cursor::MoveTo(i, 5.try_into().unwrap()),
        style::PrintStyledContent(format!("Score : {}", g.score).bold().white()),
        cursor::MoveTo(i, 6.try_into().unwrap()),
        style::PrintStyledContent(format!("Level : {}", level(&g)).bold().white()),
        cursor::MoveTo(i, 8.try_into().unwrap()),
        style::PrintStyledContent(format!("Shape : {}.{}", g.p, g.r).bold().white()),
    )
    .ok();
}

fn draw_screen(g: &Game) -> Result<()> {
    let mut stdout = stdout();

    for (i, row) in g.board.iter().enumerate() {
        let i: u16 = (i.try_into()).unwrap();

        crossterm::queue!(stdout, cursor::MoveTo(1, i + 1))?;
        row.iter()
            .enumerate()
            .map(|(j, v)| {
                let v = *v as u32;
                let j: u16 = j.try_into().unwrap();
                crossterm::queue!(stdout, cursor::MoveTo(j * 2 + 1, i + 1)).ok();
                if v != 0 {
                    let s = match v {
                        // 1 => "\u{16A0}\u{16A0}".on_red(),
                        // 2 => "\u{16A2}\u{16A2}".on_red(),
                        // 3 => "\u{16A5}\u{16A5}".on_red(),
                        // 4 => "\u{16A6}\u{16A6}".on_red(),
                        // 5 => "\u{16BC}\u{16BC}".on_red(),
                        // 6 => "\u{16AD}\u{16AD}".on_red(),
                        // _ => "\u{16D2}\u{16D2}".on_red(),
                        1 => "●●".on_blue(),
                        2 => "◎◎".blue().on_yellow(),
                        3 => "□□".on_green(),
                        4 => "◦◦".on_magenta(),
                        5 => "○○".on_dark_red(),
                        6 => "◼◼".on_cyan(),
                        _ => "◉◉".on_red(),
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
                    )
                    .ok();
                } else {
                    crossterm::queue!(stdout, style::PrintStyledContent("  ".white())).ok();
                }
            })
            .for_each(drop);
    }
    render_game_info(g);
    stdout.flush()?;
    Ok(())
}

// place a tetramino on the board
fn set_piece(g: &mut Game, x: u8, y: u8, r: u8, v: u8) {
    for i in 0..4 {
        g.board[(num(g.p, r, i * 4) + y) as usize][(num(g.p, r, i * 4 + 2) + x) as usize] = v;
    }
}

// move a piece from old (p*) coords to new
fn update_piece(g: &mut Game) {
    set_piece(g, g.px, g.py, g.pr, 0);
    g.px = g.x;
    g.py = g.y;
    g.pr = g.r;
    set_piece(g, g.x, g.y, g.r, g.p + 1);
}

fn wipe_filled_rows(g: &mut Game) {
    for row in g.y..=g.y + height(g.p, g.r) {
        if g.board[row as usize]
            .iter()
            .map(|v| *v as u32)
            //.fold(1, |p, v| p * v)
            .product::<u32>()
            > 0
        {
            for i in (1..row).rev() {
                let i = i as usize;
                for j in 0..g.board[i + 1].len() {
                    g.board[i + 1][j] = g.board[i][j];
                }
                for j in 0..g.board[0].len() {
                    g.board[0][j] = 0;
                }
                g.score += 1;
            }
        }
    }
}

// check if placing p at (x,y,r) will hit something
fn check_hit(g: &mut Game, x: u8, y: u8, r: u8) -> bool {
    let bottom: u8 = (g.board.len() - 1).try_into().unwrap();
    if y + height(g.p, r) > bottom {
        return true;
    }
    set_piece(g, g.px, g.py, g.pr, 0);

    let hits = (0..4)
        .filter(|i| {
            g.board[(y + num(g.p, r, i * 4)) as usize][(x + num(g.p, r, i * 4 + 2)) as usize] != 0
        })
        .count();
    set_piece(g, g.px, g.py, g.pr, g.p + 1);
    hits > 0
}

fn do_tick(g: &mut Game) -> bool {
    if g.paused {
        return true;
    }
    g.tick = (g.tick + 1) % std::u64::MAX;
    if g.tick % 30 <= g.tick / TICK_LEVEL {
        // only update some of the time...
        if check_hit(g, g.x, g.y + 1, g.r) {
            if g.y == 0 {
                // overflow - game over
                return false;
            }
            wipe_filled_rows(g);
            new_tetramino(g);
        } else {
            g.y += 1;
            update_piece(g);
        }
    }
    true
}

fn runloop(g: &mut Game) -> Result<()> {
    while do_tick(g) {
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
                    if g.x > 0 && !check_hit(g, g.x - 1, g.y, g.r) {
                        g.x -= 1;
                    }
                }
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                })) => {
                    if g.x + width(g.p, g.r) < 9 && !check_hit(g, g.x + 1, g.y, g.r) {
                        g.x += 1;
                    }
                }
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                })) => {
                    while !check_hit(g, g.x, g.y + 1, g.r) {
                        g.y += 1;
                        update_piece(g);
                    }
                    wipe_filled_rows(g);
                    new_tetramino(g);
                }
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                })) => {
                    g.r = (g.r + 1) % 4;
                    while g.x + width(g.p, g.r) > 9 {
                        g.x -= 1;
                    }
                    if check_hit(g, g.x, g.y, g.r) {
                        g.x = g.px;
                        g.r = g.pr;
                    }
                }
                _ => (),
            }
        }
        update_piece(g);
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

    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut game = Game {
        x: 0,
        y: 0,
        r: 0,
        pr: 0,
        px: 0,
        py: 0,
        p: 0,
        tick: 0,
        score: 0,
        board: [[0; 10]; 20],
        paused: false,
    };
    new_tetramino(&mut game);

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

    println!("Score: {}; Level: {}", game.score, level(&game));
    Ok(())
}
