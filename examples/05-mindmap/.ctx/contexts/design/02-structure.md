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

I'd like some blinking, like if it is shining a bit.

        *   **Question:** How long should the splash screen be displayed? Should it automatically transition to the main menu after a fixed duration (e.g., 3-5 seconds), or should it wait for user input (e.g., pressing any key) to proceed?

I'd like it to show for 5 seconds, then automatically go to main menu. No skipping.

2.  **Main Menu:**
    *   **Options:**
        *   "Play with other guy" (Player vs. Player)
        *   "Play with computer, Level 1" (Easy AI)
        *   "Play with computer, Level 2" (Medium AI)
        *   "Play with computer, Level 3" (Hard AI)
    *   **Rendering:** To be rendered using `ratatui`.
    *   **Navigation:**
        *   **Question:** How should the user navigate the menu? (e.g., Up/Down arrow keys to select, Enter to confirm; or number keys 1-4 to select directly?)

Let's use directly keys to choose: 1,2,3,4 to select.

        *   **Question:** Should there be an option to exit the game from the main menu? (e.g., 'Q' key or a dedicated "Exit" option).

Yes, add 9 to quit. ask for confirmation.

3.  **Game Board and Player Turn Display:**
    *   **Rendering:** The game board and player turn information will be rendered using `ratatui` (which uses `crossterm` as its backend).
    *   **Visuals:**
        *   **Question:** What visual style is preferred for the game board? (e.g., simple ASCII characters like `+---+`, `| X |`, `|---|`; or more elaborate box-drawing characters?)

Visually should be quite artistic, like "ascii sprites". Let's imagine a 7x7 sprite for each cell, and render board like that.

        *   **Question:** How should the current player's turn be indicated? (e.g., "Player X's Turn", "Player O's Turn", or highlighting the current player's symbol?)

A title above the boars should tell the turn. Only the player's keys are active, other player keys are not active.
A visual feedback should be provided for wrong input, like "shaking" effect, and some red flash.

4.  **Player Keybindings:**
    *   **Player 1 (X):** `Q`, `W`, `E`, `A`, `S`, `D`, `Z`, `X`, `C` (mapping to a 3x3 grid, likely top-left to bottom-right).
    *   **Player 2 (O):** `T`, `Y`, `U`, `G`, `H`, `J`, `B`, `N`, `M` (mapping to a 3x3 grid, likely top-left to bottom-right).
    *   **Input Handling:**
        *   **Question:** How should invalid key presses be handled? (e.g., pressing a key for an already occupied cell, pressing a key that doesn't correspond to the current player, or pressing a key outside the defined set). Should there be an error message, or simply ignore the input?

Wrong behaviour should do "shaking" effect, with different colors for different kind of errors.
There should also be a timer (30s max) per move. No move = lost.

        *   **Question:** Should there be a way to restart the game or return to the main menu during gameplay (e.g., an 'R' key for restart, 'M' for menu, 'Q' for quit)?

No, timeout is enough to terminate the game. Ctrl-C anyway should always be an option to abort abruptly the program.

5.  **Computer Difficulty Levels:**
    *   **Level 1 (Easy):** Random valid moves.
    *   **Level 2 (Medium):**
        *   **Proposed:** Prioritize winning moves if available.
        *   **Proposed:** Prioritize blocking opponent's winning moves if available.
        *   **Proposed:** Otherwise, make a random valid move.
    *   **Level 3 (Hard):**
        *   **Proposed:** Implement an unbeatable AI using the Minimax algorithm (or a similar search algorithm) to always choose the optimal move.
    *   **Question:** Are these proposed AI behaviors for Level 2 and 3 acceptable, or do you have specific strategies in mind for each?

These strategies are just fine.

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

I answered all the questions, now rewrite specs integrating all my indications.

<!-- answer-ab34ed77-b26a-499d-bc10-c8818ae1277f:begin { provider: 'gemini -y -m gemini-2.5-flash' }  -->
Okay, I have integrated all your indications into the game specifications. Here is the updated and more precise breakdown:

### Refined Game Specifications: Tic-Crab-Toe

1.  **Splash Screen:**
    *   **Content:** A "funny" ASCII art related to Tic-Tac-Toe, with a "blinking" or "shining" effect.
    *   **Rendering:** To be rendered using `ratatui`.
    *   **Behavior:** The splash screen will be displayed for a fixed duration of **5 seconds**, after which it will automatically transition to the main menu. There will be no option to skip the splash screen.

2.  **Main Menu:**
    *   **Options:**
        *   "1. Play with other guy" (Player vs. Player)
        *   "2. Play with computer, Level 1" (Easy AI)
        *   "3. Play with computer, Level 2" (Medium AI)
        *   "4. Play with computer, Level 3" (Hard AI)
        *   "9. Quit"
    *   **Rendering:** To be rendered using `ratatui`.
    *   **Navigation:** Users will select options directly by pressing the corresponding number key (`1`, `2`, `3`, `4`, or `9`).
    *   **Exit Confirmation:** If '9' is pressed, the game will ask for confirmation before exiting.

3.  **Game Board and Player Turn Display:**
    *   **Rendering:** The game board and player turn information will be rendered using `ratatui`.
    *   **Visuals:** The game board will be visually artistic, using "ascii sprites" where each cell is represented by a 7x7 character sprite.
    *   **Player Turn Indicator:** A title displayed above the game board will clearly indicate whose turn it is (e.g., "Player X's Turn").
    *   **Active Keys:** Only the current player's designated keys will be active for input; the other player's keys will be inactive.
    *   **Visual Feedback for Wrong Input:** Invalid moves (e.g., selecting an occupied cell, using the wrong player's key) will trigger a "shaking" effect on the screen, accompanied by a "red flash" and different colors to distinguish between types of errors.

4.  **Player Keybindings:**
    *   **Player 1 (X):** `Q`, `W`, `E`, `A`, `S`, `D`, `Z`, `X`, `C` (mapped to the 3x3 grid, typically top-left to bottom-right).
    *   **Player 2 (O):** `T`, `Y`, `U`, `G`, `H`, `J`, `B`, `N`, `M` (mapped to the 3x3 grid, typically top-left to bottom-right).
    *   **Invalid Input Handling:** As described above, wrong behavior will result in a "shaking" effect with distinct colors for different error types.
    *   **Move Timer:** Each player will have a **30-second timer** per move. If a player fails to make a move within this time, they automatically lose the game.
    *   **Game Termination:** There will be no in-game options to restart or return to the main menu. The game concludes upon a win, draw, or timeout. Users can always abruptly terminate the application using `Ctrl-C`.

5.  **Computer Difficulty Levels:**
    *   **Level 1 (Easy):** The AI will make random valid moves.
    *   **Level 2 (Medium):** The AI will prioritize winning moves if available, then prioritize blocking the opponent's winning moves if available. Otherwise, it will make a random valid move.
    *   **Level 3 (Hard):** The AI will implement an unbeatable strategy, likely using the Minimax algorithm or a similar optimal search algorithm, to always choose the best possible move.

### Proposed Project Structure

The previously proposed project structure remains suitable for these refined specifications:

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
    │   ├── splash_screen.rs# Defines the state and rendering logic for the splash screen, including animation.
    │   ├── main_menu.rs    # Defines the state and rendering logic for the main menu, including navigation and exit confirmation.
    │   └── game_screen.rs  # Defines the state and rendering logic for the active game board display, including artistic cell rendering, turn indicator, and visual feedback for errors.
    └── keybindings.rs      # Defines constants and utility functions for mapping `crossterm::event::KeyCode` to game board positions for both players.
```

This structure provides a clear separation of concerns and will facilitate modular development.

Do these updated specifications and the project structure look good to you?
<!-- answer-ab34ed77-b26a-499d-bc10-c8818ae1277f:end {}  -->

These look good.
Do you have any more question or can we mark these as definitive?

<!-- answer-85d99dc9-02a0-48a3-b775-f8ee3160fa6e:begin { provider: 'gemini -y -m gemini-2.5-flash' }  -->
These specifications are now definitive. We can proceed with the implementation based on this detailed plan.
<!-- answer-85d99dc9-02a0-48a3-b775-f8ee3160fa6e:end {}  -->

Let's rewrite them in their definitive form if you have no further questions, otherwise ask.
