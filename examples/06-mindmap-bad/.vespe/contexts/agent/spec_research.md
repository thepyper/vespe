"You are an expert software architect and Rust developer. Your task is to help define the specifications and select appropriate dependencies for a new Rust project: 'Tic-Crab-Toe'.

**Project Overview:**
The project is a command-line Tic-Tac-Toe game.

**Key Requirements:**
1.  **Game Logic:** Implement standard Tic-Tac-Toe rules (3x3 board, two players, win conditions for rows, columns, and diagonals, draw condition).
2.  **User Interface:** A Terminal User Interface (TUI) that displays the game board and prompts for player input.
3.  **Player Interaction:** Players should be able to make moves by entering coordinates (e.g., row and column numbers).
4.  **Game Flow:**
    *   Start a new game.
    *   Display the current board state.
    *   Prompt the current player for their move.
    *   Validate moves (e.g., cell not already taken, valid coordinates).
    *   Update the board.
    *   Check for win/draw conditions after each move.
    *   Announce the winner or a draw.
    *   Option to play again.
5.  **Error Handling:** Gracefully handle invalid user input (e.g., non-numeric input, out-of-bounds coordinates, already occupied cells).

**Your Deliverables:**

1.  **Detailed Specifications:**
    *   **Game States:** Enumerate all possible game states (e.g., `Playing`, `Win`, `Draw`, `Quit`).
    *   **Data Structures:** Suggest appropriate Rust data structures for the game board and player representation.
    *   **Core Functions/Modules:** Outline the main functions or modules required for game logic, TUI rendering, and input handling.
    *   **Input/Output:** Describe how player input will be received and how the game board and messages will be displayed in the TUI.

2.  **Dependency Selection:**
    *   Recommend a minimal set of Rust crates (dependencies) suitable for this project, specifically focusing on TUI libraries and any other utilities that would simplify development.
    *   For each recommended crate, provide a brief justification for its choice, considering factors like popularity, ease of use, active maintenance, and suitability for a simple Tic-Tac-Toe TUI.
    *   Explicitly consider TUI libraries like `crossterm`, `termion`, `tui-rs` (now `ratatui`), or similar.

Please present your findings in a clear, structured, and concise manner."

