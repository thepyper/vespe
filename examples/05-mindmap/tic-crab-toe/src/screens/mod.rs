pub mod splash_screen;

pub enum CurrentScreen {
    SplashScreen(splash_screen::SplashScreen),
    MainMenu,
    // Add other screens later
}