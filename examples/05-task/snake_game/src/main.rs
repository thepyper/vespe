use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{io, time::{Duration, Instant}};

mod game;
mod ui;

use crate::game::{Direction, GameState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut game_state = GameState::new(terminal.size()?.width, terminal.size()?.height);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(150);

    loop {
        terminal.draw(|f| ui::draw(f, &game_state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        if game_state.game_over {
                            game_state =
                                GameState::new(terminal.size()?.width, terminal.size()?.height);
                        }
                    }
                    KeyCode::Up => game_state.change_direction(Direction::Up),
                    KeyCode::Down => game_state.change_direction(Direction::Down),
                    KeyCode::Left => game_state.change_direction(Direction::Left),
                    KeyCode::Right => game_state.change_direction(Direction::Right),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if !game_state.game_over {
                game_state.tick();
            }
            last_tick = Instant::now();
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}