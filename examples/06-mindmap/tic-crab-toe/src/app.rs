use crate::screens::{CurrentScreen, splash_screen::SplashScreen};
use crate::event::AppEvent;
use crossterm::event::KeyCode;

/// Represents the application's global state.
#[derive(Debug)]
pub struct App {
    /// The current active screen of the application.
    pub current_screen: CurrentScreen,
    /// A flag to signal the application to exit.
    pub should_quit: bool,
    /// The currently selected option in the main menu.
    pub menu_selection: u8,
}

impl App {
    /// Creates a new `App` instance with initial state.
    pub fn new() -> Self {
        Self {
            current_screen: CurrentScreen::SplashScreen(SplashScreen::new()),
            should_quit: false,
            menu_selection: 1, // Default to the first option
        }
    }

    /// Updates the application's state based on the received event.
    pub fn update(&mut self, event: AppEvent) {
        match &mut self.current_screen {
            CurrentScreen::SplashScreen(splash_screen) => {
                splash_screen.update_animation();
                if splash_screen.is_finished() {
                    self.current_screen = CurrentScreen::MainMenu;
                }
            }
            _ => {}
        }
        match event {
            AppEvent::Input(key_event) => {
                match key_event.code {
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }
            AppEvent::Tick => {
                // Handle periodic updates here, e.g., for animations or timers
            }
        }
    }
}
