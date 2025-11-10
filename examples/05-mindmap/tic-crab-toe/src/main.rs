use std::{io, time::Duration};
use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

// A simple macro for deferring cleanup actions.
// In a real application, consider using a more robust error handling strategy
// or a dedicated library for resource management.
macro_rules! defer {
    ($($body:tt)*) => {
        struct Defer<F: FnOnce()>(Option<F>);
        impl<F: FnOnce()> Drop for Defer<F> {
            fn drop(&mut self) {
                if let Some(f) = self.0.take() {
                    f();
                }
            }
        }
        let _defer = Defer(Some(|| { $($body)* }));
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    // Ensure terminal is cleaned up on exit, even if errors occur
    defer! {
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    // Main application loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(size);

            let block = Block::default().title("Tic-Crab-Toe").borders(Borders::ALL);
            let greeting = Paragraph::new("Hello, Tic-Crab-Toe! Press any key to exit.");
            f.render_widget(block, size);
            f.render_widget(greeting, chunks[0]);
        })?;

        // Event handling
        if event::poll(Duration::from_millis(100))? {
            if let CrosstermEvent::Key(key) = event::read()? {
                if KeyCode::Char('q') == key.code { // For now, 'q' to quit
                    break;
                }
                // Any key press will exit for this initial test
                break;
            }
        }
    }

    Ok(())
}
