Okay, I will prepare a report summarizing the decision to use `ratatui` and `crossterm` for the Tic-Crab-Toe implementation.

### Implementation Choice Report: Tic-Crab-Toe TUI

**Selected Approach:** Declarative TUI with `ratatui` (using `crossterm` as its backend).

**Why this choice?**

The primary motivation for selecting `ratatui` over a purely `crossterm`-based approach stems from its ability to provide a higher-level, more structured, and declarative way to build Terminal User Interfaces (TUIs). While `crossterm` offers essential low-level terminal control, `ratatui` significantly streamlines the development of interactive terminal applications, even for a relatively simple game like Tic-Crab-Toe.

Key advantages that led to this decision include:

*   **Cleaner Code and Separation of Concerns:** `ratatui` promotes a clear separation between the game's core logic and its user interface rendering. This leads to more maintainable and understandable code, as UI components can be defined declaratively without intertwining with game state mutations.
*   **Easier UI Management:** With `ratatui`'s widget system and layout capabilities, displaying the game board, player turns, messages, and input prompts becomes much more straightforward. It abstracts away the complexities of cursor positioning, styling, and efficient screen updates that would otherwise need to be managed manually with `crossterm`.
*   **Scalability and Future-Proofing:** Although Tic-Crab-Toe is a simple game, adopting a robust TUI framework like `ratatui` establishes a good architectural pattern. Should the project evolve or new TUI features be desired in the future, `ratatui` provides a solid foundation that can scale more gracefully than a purely low-level implementation.
*   **Well-Supported Ecosystem:** Both `crossterm` and `ratatui` are actively maintained and have strong community backing within the Rust ecosystem. `ratatui` is the spiritual successor to the widely popular `tui-rs`, ensuring its long-term viability and access to resources and examples.

**How `ratatui` and `crossterm` will be utilized:**

*   **`crossterm` as the Foundation:** `crossterm` will serve as the essential low-level backend, handling direct interaction with the terminal. This includes enabling raw mode, reading keyboard events, and executing fundamental terminal commands. `ratatui` leverages `crossterm` for these underlying operations.
*   **`ratatui` for Declarative UI:** The application's user interface will be constructed using `ratatui`'s declarative API. This involves:
    *   Defining an `App` struct to hold the entire application state, including the game board, current player, game state (playing, win, draw, menu), and any UI-specific elements like a selected cell or messages.
    *   Creating a dedicated `ui.rs` module (or similar) that contains functions responsible for rendering the `App` state into `ratatui` widgets (e.g., `Block`, `Paragraph`, `Table`) and arranging them within the terminal frame.
    *   Utilizing `ratatui`'s rendering loop, which efficiently draws the defined UI to the terminal based on state changes.
*   **Input Handling:** Player input (e.g., selecting a cell, quitting the game) will be captured via `crossterm` events. These events will then be processed by methods within the `App` struct to update the game's internal state, which in turn triggers `ratatui` to re-render the UI.
*   **Game Logic:** Core game rules (e.g., `make_move`, `check_win`, `check_draw`) will reside in a separate `game_logic.rs` module, ensuring a clean separation from the UI layer.

This combination provides a powerful yet ergonomic development experience, allowing for efficient TUI creation while maintaining a clear and modular project structure.
