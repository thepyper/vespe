use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::io::{stdout, Stdout};

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // run app
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame| {
            frame.render_widget(
                Paragraph::new("Hello, Vespe TUI!")
                    .white()
                    .on_blue()
                    .bold()
                    .block(Block::default().title("Vespe TUI").borders(Borders::ALL)),
                frame.size(),
            );
        })?;
        should_quit = handle_events()?;
    }

    // restore terminal
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn handle_events() -> Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}