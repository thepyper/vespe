use std::{io::{self, stdout}, error::Error};
use crossterm::{terminal::{enable_raw_mode, enter_alternate_screen, leave_alternate_screen, disable_raw_mode}, execute, event::{EnableMouseCapture, DisableMouseCapture}};
use ratatui::{Terminal, prelude::{CrosstermBackend, Rect, Stylize}, widgets::Paragraph, Frame};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    execute!(stdout(), enter_alternate_screen(), EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        leave_alternate_screen(),
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: CrosstermBackend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f))?;

        // In a real application, you would handle events here
        // For now, we just exit after one draw
        break;
    }
    Ok(())
}

fn ui(frame: &mut Frame) {
    let size = frame.size();
    let greeting = Paragraph::new("Hello, Ratatui!").white().on_blue();
    frame.render_widget(greeting, size);
}
