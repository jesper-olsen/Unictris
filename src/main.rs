use std::io::Result;

mod game;
mod shape;
mod tui;

fn main() -> Result<()> {
    let mut game = game::Game::default();

    let tg = tui::TerminalGuard::new()?;
    tui::runloop(&mut game)?;
    drop(tg);

    println!("Score: {}; Level: {}", game.score, game.level());
    Ok(())
}
