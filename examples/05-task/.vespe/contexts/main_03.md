I want to create a little snake game with a TUI, in rust.
I want to use ratatui.

@set { provider: 'gemini -y -m gemini-2.5-pro' } 

<!-- task-98e7cc97-dd56-4ef1-9f61-3591b98e5834:begin {}  -->

Propose a step-by-step develompent plan.
<!-- answer-edbb6d8e-ea84-4f03-b3cf-7e3b088c4e25:begin { provider: 'gemini -y -m gemini-2.5-flash' }  -->
Here's a step-by-step development plan for creating the Snake TUI game with Rust and `ratatui`:
<!-- task-98e7cc97-dd56-4ef1-9f61-3591b98e5834:end {}  -->

1.  **Project Initialization**: Create a new Rust project and add `ratatui`, `crossterm` (for terminal backend), and `rand` (for random number generation) as dependencies.

<!-- answer-3c2fcda1-d417-4e69-bfc0-83644c4ec3b0:begin { provider: 'gemini -y -m gemini-2.5-pro' }  -->
<!-- answer-3c2fcda1-d417-4e69-bfc0-83644c4ec3b0:end {}  -->

2.  **Basic TUI Setup**: Implement the main application loop using `crossterm` and `ratatui` to draw a simple border and a "Snake Game" title.
3.  **Define Game State**: Create data structures to represent the snake (e.g., a `Vec` of coordinates), food (a single coordinate), game area dimensions, current direction, score, and game over status.
4.  **Render Game Elements**: Draw the snake, food, and game boundaries on the terminal using `ratatui` widgets.
5.  **Handle User Input**: Implement event handling to capture arrow key presses for changing the snake's direction.
6.  **Implement Game Logic (Movement)**: Update the snake's position based on its current direction in each game tick.
7.  **Food Interaction**:
    *   Generate food at a random, unoccupied position.
    *   Detect when the snake head collides with food.
    *   When food is eaten, increase the snake's length and update the score.
    *   Generate new food.
8.  **Collision Detection**:
    *   Check for collisions with the game boundaries.
    *   Check for collisions with the snake's own body.
    *   Trigger "Game Over" on collision.
9.  **Game Over Screen & Restart**: Display a "Game Over" message with the final score and provide an option to restart the game.
10. **Score Display**: Continuously display the current score.
11. **Game Speed**: Introduce a mechanism to control the game speed, potentially increasing it as the score grows.
12. **Refinement and Polish**: Add any additional features, improve UI/UX, and ensure robust error handling.
<!-- answer-edbb6d8e-ea84-4f03-b3cf-7e3b088c4e25:end {}  -->


