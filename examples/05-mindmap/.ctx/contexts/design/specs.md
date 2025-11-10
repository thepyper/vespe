Okay, I understand. Let's define the project structure and then refine the game specifications.

### Proposed Project Structure

Based on the decision to use `ratatui` and `crossterm`, and the features you've outlined, here's a proposed modular Rust project structure:

```
tic-crab-toe/
├── Cargo.toml
├── src/
│   ├── main.rs             # Application entry point, main event loop, TUI setup/teardown.
│   ├── app.rs              # Manages the overall application state and transitions between screens (splash, menu, game).
│   ├── ui.rs               # Handles rendering logic for different application states using ratatui.
│   ├── tui.rs              # Low-level terminal interaction (crossterm setup, event reading).
│   ├── game/               # Core game logic module
│   │   ├── mod.rs          # Defines GameState, Board, Player, Cell, etc.
│   │   └── logic.rs        # Implements game rules (make_move, check_win, check_draw).
│   ├── menu/               # Main menu module
│   │   ├── mod.rs          # Defines menu options and state.
│   │   └── ui.rs           # Renders the menu using ratatui.
│   ├── splash/             # Splash screen module
│   │   ├── mod.rs          # Defines splash screen state (e.g., animation frames).
│   │   └── ui.rs           # Renders the splash screen using ratatui.
│   └── ai/                 # Artificial Intelligence module for computer opponents
│       ├── mod.rs          # Defines AI player types and common AI traits.
│       ├── easy.rs         # Implements AI logic for Level 1.
│       ├── medium.rs       # Implements AI logic for Level 2.
│       └── hard.rs         # Implements AI logic for Level 3.
└── assets/                 # Directory for static assets like ASCII art
    └── splash_ascii.txt    # ASCII art for the splash screen.
```

### Refined Specifications and Clarification Questions

Here are your specifications, refined for clarity, along with questions to gather any missing details:

1.  **Splash Screen:**
    *   **Requirement:** A funny splash screen rendered with `ratatui`, featuring fancy ASCII art related to Tic-Tac-Toe.
    *   **Clarification Questions:**
        *   **Duration:** Should the splash screen be displayed for a fixed duration (e.g., 3-5 seconds), or should it wait for user input (e.g., pressing any key) to proceed to the main menu?
        *   **ASCII Art:** Do you have specific ASCII art in mind, or should I source/generate a suitable one?

2.  **Main Menu:**
    *   **Requirement:** After the splash screen, display a main menu with the following options:
        *   "Play with other guy" (Human vs. Human)
        *   "Play with computer, level 1" (Human vs. Easy AI)
        *   "Play with computer, level 2" (Human vs. Medium AI)
        *   "Play with computer, level 3" (Human vs. Hard AI)
    *   **Clarification Questions:**
        *   **Navigation:** How should the user navigate between menu options? (e.g., Up/Down arrow keys, 'w'/'s' keys, or number selection?)
        *   **Selection:** What key should be used to select a highlighted menu option? (e.g., Enter key?)

3.  **Game Board and Player's Turn Display:**
    *   **Requirement:** Upon selecting a game mode, display the game board and clearly indicate the current player's turn.
    *   **Clarification Questions:**
        *   **Board Visuals:** What should the visual style of the game board be? (e.g., a simple text-based grid like `+---+---+---+`, or a more elaborate `ratatui` block layout with borders and cell highlighting?)
        *   **Turn Indicator:** How should the current player's turn be visually indicated? (e.g., a text message like "Player X's Turn", highlighting the player's symbol, or a distinct color for the active player?)
        *   **Game Outcome:** How should the game's outcome (Win, Lose, Draw) be displayed to the user? Should there be a "Play Again" option, or return to the main menu automatically?

4.  **Keybindings for Player Moves:**
    *   **Requirement:**
        *   Player 1 keys: `Q`, `W`, `E`, `A`, `S`, `D`, `Z`, `X`, `C` (corresponding to a 3x3 grid on an Italian keyboard).
        *   Player 2 keys: `T`, `Y`, `U`, `G`, `H`, `J`, `B`, `N`, `M` (corresponding to a 3x3 grid on an Italian keyboard).
    *   **Clarification Questions:**
        *   **General Actions:** What keys should be reserved for general application actions, such as:
            *   Quitting the game/application? (e.g., `Esc`, `q`, `Ctrl+C`)
            *   Returning to the main menu from a game? (e.g., `m`)
            *   Restarting the current game? (e.g., `r`)
        *   **Case Sensitivity:** Should the player input keys be case-sensitive (e.g., only 'q' not 'Q') or case-insensitive?

5.  **Computer Difficulty Levels:**
    *   **Requirement:** The computer opponent should have increasing difficulty across levels 1, 2, and 3.
    *   **Clarification Questions:**
        *   **Level 1 (Easy):** What specific behavior should the easy AI exhibit? (e.g., always make a random valid move?)
        *   **Level 2 (Medium):** What specific behavior should the medium AI exhibit? (e.g., block immediate player wins, but otherwise make random moves? Or prioritize its own immediate wins?)
        *   **Level 3 (Hard):** What specific behavior should the hard AI exhibit? (e.g., optimal play using an algorithm like Minimax, or a very strong heuristic-based approach?)

Please provide your answers to these questions so I can proceed with the implementation plan.
