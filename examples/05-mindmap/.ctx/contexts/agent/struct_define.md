Please provide a prompt for an agent to define the project structure for a Rust TUI game named "Tic-Crab-Toe". The game will use `ratatui` for the declarative UI and `crossterm` as its backend for low-level terminal control and event handling.

The project structure should accommodate the following components:
*   An `App` struct to manage the overall application state (game board, current player, game state, UI elements).
*   A dedicated module (e.g., `ui.rs`) for rendering the `App` state using `ratatui` widgets.
*   A separate module (e.g., `game_logic.rs`) for the core game rules (making moves, checking for wins/draws).
*   Input handling, which will involve processing `crossterm` events and updating the `App` state.

The agent should propose a clear, modular, and idiomatic Rust project structure, including relevant directories and files.
