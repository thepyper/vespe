I want to create a little snake game with a TUI, in rust.

<!-- task-b91c2c0c-2694-48ff-bee2-d70e91bf97c1:begin {}  -->
<!-- task-b91c2c0c-2694-48ff-bee2-d70e91bf97c1:end {}  -->

Propose a step-by-step develompent plan.
<!-- answer-7a98105b-00d0-47b9-a144-d84b898614f1:begin { provider: 'gemini -y -m gemini-2.5-flash' }  -->
Here's a step-by-step development plan for your TUI Snake game in Rust:

1.  **Project Initialization**: Create a new Rust project using `cargo new snake_tui`.
2.  **Dependency Management**: Add a TUI library (e.g., `crossterm`) and potentially a random number generator (`rand`) to `Cargo.toml`.
3.  **Game State Definition**:
    *   Define structs for `Snake` (body segments, direction), `Food` (position), `Game` (board dimensions, score, game over status).
    *   Implement methods for `Snake` movement, growth, and collision checks.
4.  **TUI Setup**:
    *   Initialize `crossterm` for raw mode, hide cursor, and clear screen.
    *   Set up event polling for keyboard input.
5.  **Main Game Loop**:
    *   A continuous loop that:
        *   Handles user input to change snake direction or quit.
        *   Updates the game state (move snake, check collisions, generate food).
        *   Renders the game board, snake, food, and score to the terminal.
        *   Introduces a small delay to control game speed.
6.  **Input Handling**:
    *   Process `crossterm` events for arrow keys (Up, Down, Left, Right) to control the snake.
    *   Handle 'q' or 'Esc' for quitting the game.
7.  **Rendering Logic**:
    *   Clear the terminal screen.
    *   Draw borders for the game area.
    *   Draw the snake segments.
    *   Draw the food.
    *   Display the current score and "Game Over" message if applicable.
8.  **Game Logic Implementation**:
    *   **Movement**: Update snake head position based on current direction, shift body segments.
    *   **Food**: Generate food at a random valid position. Detect when the snake eats food, increase score, and grow the snake.
    *   **Collision Detection**: Check for collisions with walls and the snake's own body.
    *   **Game Over**: Set game over flag on collision.
9.  **Cleanup**: Restore terminal to original state on exit.
10. **Testing**: Implement basic unit tests for core game logic components (e.g., snake movement, collision detection).
11. **Refinement**: Add features like difficulty levels, pause functionality, or improved aesthetics.
<!-- answer-7a98105b-00d0-47b9-a144-d84b898614f1:end {}  -->


