@include project

Define a good system prompt for the agent that will help me defining the implementation plan.
That is, the order in which is best to approach implementation.

<!-- answer-0d3132c9-9ccb-46ec-b24b-2060c40a9279:begin {
	output: agent/planner,
	provider: 'gemini -y -m gemini-2.5-flash'
}  -->
<!-- answer-0d3132c9-9ccb-46ec-b24b-2060c40a9279:end {}  -->

@forget

@include project

@set {
    provider:  'gemini -y -m gemini-2.5-flash',
    prefix: agent/planner
}

Let's propose and justify one (or more if you have many good options) plan to approach project implementation.
I want to maximize success probability, so maybe a small step-by-small step approach is best?
What do you think?

<!-- answer-ad0e4b74-53ca-4c6f-be0e-bcecf4198e8a:begin {
	provider: 'gemini -y -m gemini-2.5-flash',
	prefix: agent/planner
}  -->
Here is a detailed, step-by-step implementation plan for the "Tic-Crab-Toe" game, prioritizing foundational elements, iterative development, clear dependencies, test-driven approach, separation of concerns, and complexity management.

---

### Tic-Crab-Toe Implementation Plan

This plan outlines the development process in a series of manageable steps, ensuring a robust foundation before tackling more complex features.

1.  **Initial Project Setup and Basic Terminal Application**
    *   **What:** Initialize the Rust project, add `crossterm` and `ratatui` dependencies, and create a minimal `main.rs` that sets up the terminal, enters raw mode, and runs an empty `ratatui` rendering loop. This step aims to get a blank `ratatui` canvas working.
    *   **Why:** This is the absolute foundational step. It verifies that the chosen TUI libraries are correctly integrated and that the application can interact with the terminal environment. All subsequent UI and event handling depends on this.
    *   **Which modules:** `Cargo.toml`, `src/main.rs`.
    *   **Key considerations:** Handling `crossterm`'s raw mode and event polling, `ratatui`'s `Terminal` setup, and basic error handling for terminal operations.

2.  **Define Core Application State and Event Handling Infrastructure**
    *   **What:** Define the `App` struct in `src/app.rs` to hold the overall application state (e.g., `current_screen`, `game_state`, `menu_selection`). Implement a basic event polling mechanism in `src/event.rs` to capture `crossterm` events and translate them into a custom `AppEvent` enum (e.g., `Tick`, `Input(KeyEvent)`). Integrate this into the `main.rs` loop to update the `App` state.
    *   **Why:** Establishes the central state management and event dispatching system. All UI rendering and game logic will react to changes in the `App` state, driven by `AppEvent`s. This is critical for modularity and responsiveness.
    *   **Which modules:** `src/app.rs`, `src/event.rs`, `src/main.rs`.
    *   **Key considerations:** Designing a flexible `AppEvent` enum, ensuring efficient event polling without blocking the UI, and defining initial `CurrentScreen` states (e.g., `Loading`, `MainMenu`).

3.  **Implement Splash Screen**
    *   **What:** Create `src/screens/splash_screen.rs` to manage the splash screen's state (e.g., animation frame, timer). Implement rendering logic in `src/ui.rs` to display the "funny" ASCII art with a "blinking" or "shining" effect using `ratatui` widgets. The `App` will transition from `SplashScreen` to `MainMenu` after 5 seconds, managed by `AppEvent::Tick` and the splash screen's internal timer.
    *   **Why:** This is the first visible UI component and validates the `ratatui` rendering pipeline, screen transitions, and time-based event handling. It provides immediate visual feedback on progress.
    *   **Which modules:** `src/screens/mod.rs`, `src/screens/splash_screen.rs`, `src/ui.rs`, `src/app.rs`, `src/event.rs`.
    *   **Key considerations:** Implementing the animation effect (e.g., cycling through different ASCII art states), managing the 5-second timer, and ensuring a smooth transition to the next screen.

4.  **Implement Main Menu**
    *   **What:** Create `src/screens/main_menu.rs` to define the menu options and handle user selection. Implement rendering logic in `src/ui.rs` to display the menu using `ratatui` widgets. The `App` will process number key inputs (`1`, `2`, `3`, `4`, `9`) to navigate or initiate game modes. Implement an exit confirmation dialog for the '9' key.
    *   **Why:** This enables user interaction and allows for selecting different game modes. It further validates input handling, screen transitions, and basic dialog management.
    *   **Which modules:** `src/screens/mod.rs`, `src/screens/main_menu.rs`, `src/ui.rs`, `src/app.rs`, `src/event.rs`.
    *   **Key considerations:** Mapping key presses to menu actions, managing the selected menu item (if visual selection is desired), and implementing the two-step exit confirmation.

5.  **Develop Core Game Logic**
    *   **What:** Implement the `Board`, `Player` (X/O), `Cell`, `GameStatus` (Playing, Win, Draw), and `Move` structs in `src/game/mod.rs`. Develop core game functions: `new_game()`, `make_move()`, `check_win()`, `check_draw()`, and `get_available_moves()`. This module should be purely logical, without any UI dependencies.
    *   **Why:** This establishes the fundamental rules and state of the game. It's a critical backend component that can be developed and unit-tested independently of the UI, ensuring correctness before integration.
    *   **Which modules:** `src/game/mod.rs`.
    *   **Key considerations:** Designing clear data structures for the game state, implementing efficient win/draw checking algorithms, and ensuring all game rules are correctly enforced.

6.  **Define Player Keybindings**
    *   **What:** Create `src/keybindings.rs` to define constants and utility functions for mapping `crossterm::event::KeyCode` to game board positions (0-8) for both Player 1 (X) and Player 2 (O).
    *   **Why:** This centralizes the input mapping logic, making it easy to modify and test. It's a prerequisite for integrating player input with the game logic.
    *   **Which modules:** `src/keybindings.rs`.
    *   **Key considerations:** Clear and unambiguous mapping for all 9 cells for both players, and a function to easily retrieve the board position from a `KeyCode`.

7.  **Implement Basic Game Screen Rendering**
    *   **What:** Create `src/screens/game_screen.rs` to manage the game screen's state. Implement rendering logic in `src/ui.rs` to display the 3x3 game board. Initially, use simple character representations for cells (e.g., 'X', 'O', ' '). Display a title indicating the current player's turn (e.g., "Player X's Turn").
    *   **Why:** This connects the core game logic (`src/game/mod.rs`) to the UI. It allows for visual verification of game state changes and player turns.
    *   **Which modules:** `src/screens/mod.rs`, `src/screens/game_screen.rs`, `src/ui.rs`, `src/app.rs`.
    *   **Key considerations:** Laying out the 3x3 grid, dynamically updating cell content based on `game_state`, and displaying the current player.

8.  **Player vs. Player Game Flow**
    *   **What:** Integrate player input from `src/event.rs` and `src/keybindings.rs` with the `App`'s game state. When a valid key is pressed, call `game::make_move()` and update the UI. Handle invalid moves by ignoring input and providing basic feedback (e.g., a console message for now). Implement game termination conditions (win/draw) and transition to a simple "Game Over" screen or back to the main menu.
    *   **Why:** This completes the core interactive gameplay loop for two human players. It validates the full cycle of input -> state update -> UI render -> game outcome.
    *   **Which modules:** `src/app.rs`, `src/event.rs`, `src/keybindings.rs`, `src/game/mod.rs`, `src/screens/game_screen.rs`, `src/ui.rs`.
    *   **Key considerations:** Ensuring correct turn-taking, validating moves against `game_state`, and accurately detecting win/draw conditions.

9.  **Implement Easy AI (Random Moves)**
    *   **What:** Create `src/game/ai.rs`. Implement the "Easy" AI logic: select a random valid move from `game::get_available_moves()`. Integrate this AI into the `App`'s game loop when "Play with computer, Level 1" is selected.
    *   **Why:** Introduces the first AI opponent, validating the AI module structure and its interaction with the core game logic. It's the simplest AI to implement, allowing for early testing of AI integration.
    *   **Which modules:** `src/game/ai.rs`, `src/app.rs`, `src/main_menu.rs`.
    *   **Key considerations:** Ensuring the AI only makes valid moves and correctly integrates into the turn-based system.

10. **Implement Medium AI (Basic Strategy)**
    *   **What:** Enhance `src/game/ai.rs` with "Medium" AI logic:
        1.  Check for immediate winning moves for itself.
        2.  Check for immediate winning moves for the opponent and block.
        3.  If neither, make a random valid move.
    *   **Why:** Increases the complexity of the AI, demonstrating a more strategic opponent. This step builds upon the basic AI structure and introduces decision-making logic.
    *   **Which modules:** `src/game/ai.rs`.
    *   **Key considerations:** Efficiently checking for winning and blocking moves, and prioritizing these over random moves.

11. **Implement Hard AI (Minimax Algorithm)**
    *   **What:** Implement the "Hard" AI using the Minimax algorithm in `src/game/ai.rs`. This AI should be unbeatable.
    *   **Why:** Provides the most challenging opponent and completes the AI functionality. This is the most complex AI, so it's placed later in the plan.
    *   **Which modules:** `src/game/ai.rs`.
    *   **Key considerations:** Correct implementation of Minimax (or a similar optimal search algorithm), handling game states, and optimizing performance if necessary.

12. **Implement Artistic Game Board and Visual Feedback**
    *   **What:** Refine `src/screens/game_screen.rs` and `src/ui.rs` to render the game board with "artistic ascii sprites" for each 7x7 cell. Implement the "shaking" effect and "red flash" with different color indicators for invalid moves.
    *   **Why:** Enhances the user experience and provides crucial visual cues. This step adds polish and addresses specific UI requirements.
    *   **Which modules:** `src/screens/game_screen.rs`, `src/ui.rs`, `src/app.rs`.
    *   **Key considerations:** Designing the ASCII sprites, implementing the shaking effect (e.g., by slightly shifting UI elements or using `ratatui`'s `Modifier`s), and dynamically changing colors for error feedback.

13. **Implement Move Timer**
    *   **What:** Add a 30-second timer to `src/app.rs` and `src/screens/game_screen.rs` for each player's turn. If the timer expires, the current player forfeits the game. Display the remaining time on the game screen.
    *   **Why:** Adds a critical gameplay mechanic and further validates time-based event handling and game state changes.
    *   **Which modules:** `src/app.rs`, `src/event.rs`, `src/screens/game_screen.rs`, `src/ui.rs`.
    *   **Key considerations:** Accurately tracking time, updating the UI with the remaining time, and handling the forfeiture condition.

14. **Comprehensive Testing and Refinement**
    *   **What:** Write unit tests for `src/game/mod.rs` and `src/game/ai.rs` to ensure game logic and AI strategies are correct. Conduct integration testing for UI components, event handling, and screen transitions. Address any bugs, performance issues, or UI glitches.
    *   **Why:** Ensures the overall quality, stability, and correctness of the application. This final phase is crucial for delivering a polished product.
    *   **Which modules:** All modules.
    *   **Key considerations:** Covering edge cases in game logic, verifying AI behavior, and ensuring a smooth and responsive user experience.

---
<!-- answer-ad0e4b74-53ca-4c6f-be0e-bcecf4198e8a:end {}  -->

It looks like a good plan.
Now review it, I think a good requirement for a successful plan is that each step is as isolated as possible (not completely isolated of course)
from others. So, review your plan, and check if some files are touched "too much times" and you can optimize something about that.
If plan is already ok, just keep it.
Outline only the changes you can do to the plan, then we decide if changing or not.

<!-- answer-a8b3927a-f164-429b-ba6f-e777bdcf4aa8:begin {
	provider: 'gemini -y -m gemini-2.5-flash',
	prefix: agent/planner
}  -->
The current plan is well-structured for iterative development, and the observed repeated touches on core files like `src/app.rs`, `src/ui.rs`, and `src/event.rs` are natural given their central role in managing application state, rendering, and event handling in a TUI application. Each modification to these files is intended to integrate new functionality developed in that specific step, rather than indicating a lack of isolation.

Therefore, I propose no significant changes to the plan. It already maximizes success probability through a small step-by-small step approach, building foundational elements before adding complexity.
<!-- answer-a8b3927a-f164-429b-ba6f-e777bdcf4aa8:end {}  -->

