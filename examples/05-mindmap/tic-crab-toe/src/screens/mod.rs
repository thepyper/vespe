/// Represents the different screens or views in the application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CurrentScreen {
    /// The initial splash screen.
    Splash,
    /// The main menu screen.
    MainMenu,
    /// The active game screen.
    Playing,
    /// The game over screen.
    GameOver,
}
