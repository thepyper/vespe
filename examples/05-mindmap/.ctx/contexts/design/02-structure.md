@include project

@set {
    provider: 'gemini -y -m gemini-2.5-flash'
}

Write a system prompt for the agent that will help me in defining the project structure.

<!-- answer-cd101fa2-0058-42ac-800d-d187344afcb4:begin {
	provider: 'gemini -y -m gemini-2.5-flash',
	output: agent/struct_define
}  -->
<!-- answer-cd101fa2-0058-42ac-800d-d187344afcb4:end {}  -->

@forget

@include project

We need to decide project structure.
Reason step by step, and propose a project structure for the game.
I want the following things:
1. when game starts, I want a funny splash screen rendered with ratatui, with a fancy ascii art about tic tac toe.
2. after the splash animation, I want main menu, with following options:
    - play with other guy
    - play with computer, level 1
    - play with computer, level 2
    - play with computer, level 3
3. after selecting one of above option, I want to see game board and player's turn rendered in crossterm.
4. to avoid cross-selection for players, player 1 should play with these keys: QWEASDZXC (3x3 on italian keyboard), 
   and player 2 should play with these keys: TYUGHJBNM (3x3 on italian keyboard).
5. computer should have increasing difficulty in levels.

Write down these specifications in a more precise manner, ask me questions if some important info is missing.

<!-- answer-738865cc-66dd-4c64-98aa-814969dc1300:begin { provider: 'gemini -y -m gemini-2.5-flash' }  -->
Okay, this is a great starting point! The decision to use `ratatui` with `crossterm` is solid for a Rust TUI game. Let's refine the specifications and propose a project structure.

### Refined Game Specifications

Here's a more precise breakdown of your requirements, with some clarifications and questions for you:

1.  **Splash Screen:**
    *   **Content:** A "funny" ASCII art related to Tic-Tac-Toe.
    *   **Rendering:** To be rendered using `ratatui`.
    *   **Behavior:**
        *   **Question:** Should the ASCII art be static or include a simple animation (e.g., blinking text, subtle movement)?
        *   **Question:** How long should the splash screen be displayed? Should it automatically transition to the main menu after a fixed duration (e.g., 3-5 seconds), or should it wait for user input (e.g., pressing any key) to proceed?

2.  **Main Menu:**
    *   **Options:**
        *   "Play with other guy" (Player vs. Player)
        *   "Play with computer, Level 1" (Easy AI)
        *   "Play with computer, Level 2" (Medium AI)
        *   "Play with computer, Level 3" (Hard AI)
    *   **Rendering:** To be rendered using `ratatui`.
    *   **Navigation:**
        *   **Question:** How should the user navigate the menu? (e.g., Up/Down arrow keys to select, Enter to confirm; or number keys 1-4 to select directly?)
        *   **Question:** Should there be an option to exit the game from the main menu? (e.g., 'Q' key or a dedicated "Exit" option).

3.  **Game Board and Player Turn Display:**
    *   **Rendering:** The game board and player turn information will be rendered using `ratatui` (which uses `crossterm` as its backend).
    *   **Visuals:**
        *   **Question:** What visual style is preferred for the game board? (e.g., simple ASCII characters like `+---+`, `| X |`, `|---|`; or more elaborate box-drawing characters?)
        *   **Question:** How should the current player's turn be indicated? (e.g., "Player X's Turn", "Player O's Turn", or highlighting the current player's symbol?)

4.  **Player Keybindings:**
    *   **Player 1 (X):** `Q`, `W`, `E`, `A`, `S`, `D`, `Z`, `X`, `C` (mapping to a 3x3 grid, likely top-left to bottom-right).
    *   **Player 2 (O):** `T`, `Y`, `U`, `G`, `H`, `J`, `B`, `N`, `M` (mapping to a 3x3 grid, likely top-left to bottom-right).
    *   **Input Handling:**
        *   **Question:** How should invalid key presses be handled? (e.g., pressing a key for an already occupied cell, pressing a key that doesn't correspond to the current player, or pressing a key outside the defined set). Should there be an error message, or simply ignore the input?
        *   **Question:** Should there be a way to restart the game or return to the main menu during gameplay (e.g., an 'R' key for restart, 'M' for menu, 'Q' for quit)?

5.  **Computer Difficulty Levels:**
    *   **Level 1 (Easy):** Random valid moves.
    *   **Level 2 (Medium):**
        *   **Proposed:** Prioritize winning moves if available.
        *   **Proposed:** Prioritize blocking opponent's winning moves if available.
        *   **Proposed:** Otherwise, make a random valid move.
    *   **Level 3 (Hard):**
        *   **Proposed:** Implement an unbeatable AI using the Minimax algorithm (or a similar search algorithm) to always choose the optimal move.
    *   **Question:** Are these proposed AI behaviors for Level 2 and 3 acceptable, or do you have specific strategies in mind for each?

### Proposed Project Structure

Based on the refined specifications and best practices for `ratatui` applications, here's a proposed project structure:

```
tic-crab-toe/
├── Cargo.toml
└── src/
    ├── main.rs             # Application entry point. Initializes terminal, runs the main event loop, and manages top-level application state transitions.
    ├── app.rs              # Defines the core `App` struct, which holds the entire application state (game, current screen, menu selection, etc.) and methods for updating this state based on events.
    ├── ui.rs               # Contains functions responsible for rendering the `App` state into `ratatui` widgets and arranging them within the terminal frame. This module will call into screen-specific rendering logic.
    ├── event.rs            # Defines custom `AppEvent` enum (e.g., `Tick`, `Input(KeyEvent)`, `GameEvent(GameEvent)`) and handles `crossterm` events, translating them into `AppEvent`s for the main loop.
    ├── game/               # Module for core game logic and AI
    │   ├── mod.rs          # Defines `Board`, `Player` (X/O), `Cell`, `GameStatus` (Playing, Win, Draw), `Move` structs, and core game rules (make_move, check_win, check_draw).
    │   └── ai.rs           # Implements the AI logic for different difficulty levels (Random, Basic, Minimax).
    ├── screens/            # Module for managing different application screens/views
    │   ├── mod.rs          # Defines `CurrentScreen` enum (Splash, MainMenu, Playing, GameOver) and handles screen-specific state and transitions.
    │   ├── splash_screen.rs# Defines the state and rendering logic for the splash screen.
    │   ├── main_menu.rs    # Defines the state and rendering logic for the main menu.
    │   └── game_screen.rs  # Defines the state and rendering logic for the active game board display.
    └── keybindings.rs      # Defines constants and utility functions for mapping `crossterm::event::KeyCode` to game board positions for both players.
```

This structure aims for a clear separation of concerns:
*   `main.rs` for setup and the main loop.
*   `app.rs` for overall application state.
*   `ui.rs` for rendering.
*   `event.rs` for input processing.
*   `game/` for game rules and AI.
*   `screens/` for managing distinct views (splash, menu, game).
*   `keybindings.rs` for input mapping.

Please review these refined specifications and the proposed project structure. Let me know if you have any adjustments or further questions!
<!-- answer-738865cc-66dd-4c64-98aa-814969dc1300:end {}  -->
