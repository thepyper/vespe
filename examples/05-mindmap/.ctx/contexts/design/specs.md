Here are the definitive specifications for Project Tic-Crab-Toe, integrating all your feedback and decisions:

### Definitive Game Specifications: Tic-Crab-Toe

1.  **Splash Screen:**
    *   **Content:** A "funny" ASCII art related to Tic-Tac-Toe, featuring a "blinking" or "shining" animation effect.
    *   **Rendering:** To be rendered using `ratatui`.
    *   **Behavior:** The splash screen will be displayed for a fixed duration of **5 seconds**. After this duration, it will automatically transition to the main menu. There will be no option for the user to skip the splash screen.

2.  **Main Menu:**
    *   **Options:**
        *   "1. Play with other guy" (Player vs. Player)
        *   "2. Play with computer, Level 1" (Easy AI)
        *   "3. Play with computer, Level 2" (Medium AI)
        *   "4. Play with computer, Level 3" (Hard AI)
        *   "9. Quit"
    *   **Rendering:** To be rendered using `ratatui`.
    *   **Navigation:** Users will select options directly by pressing the corresponding number key (`1`, `2`, `3`, `4`, or `9`).
    *   **Exit Confirmation:** If the '9' key is pressed, the game will prompt the user for confirmation before exiting the application.

3.  **Game Board and Player Turn Display:**
    *   **Rendering:** The game board and player turn information will be rendered using `ratatui`.
    *   **Visuals:** The game board will feature an artistic design, utilizing "ascii sprites" where each individual cell is represented by a 7x7 character sprite.
    *   **Player Turn Indicator:** A clear title displayed prominently above the game board will indicate whose turn it is (e.g., "Player X's Turn").
    *   **Active Keys:** Only the designated keybindings for the current player will be active for input. Key presses corresponding to the other player will be ignored.
    *   **Visual Feedback for Wrong Input:** Invalid moves (e.g., attempting to select an already occupied cell, pressing a key not assigned to the current player) will trigger a distinct "shaking" effect on the screen. This will be accompanied by a "red flash" and different color indicators to visually distinguish between various types of input errors.

4.  **Player Keybindings:**
    *   **Player 1 (X):** `Q`, `W`, `E`, `A`, `S`, `D`, `Z`, `X`, `C`. These keys will map to the 3x3 game grid, typically from top-left to bottom-right.
    *   **Player 2 (O):** `T`, `Y`, `U`, `G`, `H`, `J`, `B`, `N`, `M`. These keys will also map to the 3x3 game grid, typically from top-left to bottom-right.
    *   **Invalid Input Handling:** As detailed above, incorrect input will result in a "shaking" effect with specific color cues for different error conditions.
    *   **Move Timer:** Each player will be allotted a **30-second timer** to make their move. If a player fails to input a valid move within this time limit, they will automatically forfeit the game.
    *   **Game Termination:** There will be no in-game options to restart the current game or return to the main menu. The game concludes upon a win, a draw, or a player's move timeout. Users can always force an abrupt termination of the application by pressing `Ctrl-C`.

5.  **Computer Difficulty Levels:**
    *   **Level 1 (Easy):** The AI will select its moves randomly from all available valid positions on the board.
    *   **Level 2 (Medium):** The AI will first check for any immediate winning moves for itself and execute one if found. If no winning move is available, it will then check if the opponent has any immediate winning moves and block one if possible. If neither of these conditions applies, the AI will make a random valid move.
    *   **Level 3 (Hard):** The AI will implement an unbeatable strategy, utilizing an optimal search algorithm such as the Minimax algorithm, to consistently choose the best possible move, ensuring it never loses.

### Proposed Project Structure

The following project structure will be adopted to ensure a clear separation of concerns and facilitate modular development:

```
tic-crab-toe/
├── Cargo.toml
└── src/
    ├── main.rs             # Application entry point. Initializes the terminal, runs the main event loop, and manages top-level application state transitions.
    ├── app.rs              # Defines the core `App` struct, which holds the entire application state (game, current screen, menu selection, etc.) and methods for updating this state based on events.
    ├── ui.rs               # Contains functions responsible for rendering the `App` state into `ratatui` widgets and arranging them within the terminal frame. This module will call into screen-specific rendering logic.
    ├── event.rs            # Defines a custom `AppEvent` enum (e.g., `Tick`, `Input(KeyEvent)`, `GameEvent(GameEvent)`) and handles `crossterm` events, translating them into `AppEvent`s for the main loop.
    ├── game/               # Module for core game logic and AI
    │   ├── mod.rs          # Defines `Board`, `Player` (X/O), `Cell`, `GameStatus` (Playing, Win, Draw), `Move` structs, and core game rules (make_move, check_win, check_draw).
    │   └── ai.rs           # Implements the AI logic for different difficulty levels (Random, Basic, Minimax).
    ├── screens/            # Module for managing different application screens/views
    │   ├── mod.rs          # Defines `CurrentScreen` enum (Splash, MainMenu, Playing, GameOver) and handles screen-specific state and transitions.
    │   ├── splash_screen.rs# Defines the state and rendering logic for the splash screen, including animation.
    │   ├── main_menu.rs    # Defines the state and rendering logic for the main menu, including navigation and exit confirmation.
    │   └── game_screen.rs  # Defines the state and rendering logic for the active game board display, including artistic cell rendering, turn indicator, and visual feedback for errors.
    └── keybindings.rs      # Defines constants and utility functions for mapping `crossterm::event::KeyCode` to game board positions for both players.
```
