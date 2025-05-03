use crossterm::{
    QueueableCommand, cursor,
    event::{Event, KeyCode, KeyEvent, poll, read},
    style::{self, Stylize},
    terminal,
};
use std::io::{Result, Write, stdout};
use std::time;

mod game;
mod shape;

use crate::game::{BOARD_HEIGHT, BOARD_WIDTH, Game, Move};

fn centered_x(s: &str) -> u16 {
    let leftedge: u16 = 25;
    let n: u16 = s.len().try_into().expect("really long string");

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
            format!(
                "Shape : {}.{}",
                g.tetromino.shape.kind(),
                g.tetromino.orientation
            )
            .bold()
            .white()
        ),
    )?;
    Ok(())
}

fn draw_screen(g: &Game) -> Result<()> {
    let mut stdout = stdout();

    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            crossterm::queue!(stdout, cursor::MoveTo(x as u16 * 2 + 1, y as u16 + 1))?;
            let s = match g.board.get(x, y) {
                0 => "  ".white(),
                // 1 => "\u{16A0}\u{16A0}".on_red(),
                // 2 => "\u{16A2}\u{16A2}".on_red(),
                // 3 => "\u{16A5}\u{16A5}".on_red(),
                // 4 => "\u{16A6}\u{16A6}".on_red(),
                // 5 => "\u{16BC}\u{16BC}".on_red(),
                // 6 => "\u{16AD}\u{16AD}".on_red(),
                // _ => "\u{16D2}\u{16D2}".on_red(),
                // 1 => "●●".on_blue(),
                // 2 => "◎◎".blue().on_yellow(),
                // 3 => "□□".on_green(),
                // 4 => "◦◦".on_magenta(),
                // 5 => "○○".on_dark_red(),
                // 6 => "◼◼".on_cyan(),
                // _ => "◉◉".on_red(),
                1 => "  ".on_blue(),
                2 => "  ".on_yellow(),
                3 => "  ".on_green(),
                4 => "  ".on_magenta(),
                5 => "  ".on_dark_red(),
                6 => "  ".on_cyan(),
                _ => "  ".on_red(),
            };
            crossterm::queue!(
                stdout,
                style::PrintStyledContent(s),
                cursor::MoveTo((x as u16 + 1) * 2 + 1, y as u16 + 1)
            )?
        }
    }
    render_game_info(g)?;
    stdout.flush()
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
    let mut game = Game::default();

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
