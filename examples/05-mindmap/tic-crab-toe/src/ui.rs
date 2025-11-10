// src/ui.rs
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::app::App;
use crate::screens::CurrentScreen;
use crate::screens::splash_screen; // Import the splash_screen module

pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
    match app.current_screen {
        CurrentScreen::SplashScreen => {
            splash_screen::render(f, app); // Call the splash screen rendering function
        }
        CurrentScreen::MainMenu => {
            // Render main menu later
        }
        // ... other screens
    }
}
