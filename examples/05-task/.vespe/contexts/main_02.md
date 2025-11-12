I want to create a little snake game with a TUI, in rust.
I want to use ratatui.

@set { provider: 'gemini -y -m gemini-2.5-pro' } 

<!-- task-768e3075-2008-4eac-aa64-687b91c04ba8:begin { provider: 'gemini -y -m gemini-2.5-pro' }  -->
<!-- task-768e3075-2008-4eac-aa64-687b91c04ba8:end {}  -->

Propose a step-by-step develompent plan.
<!-- answer-87151488-88d3-4648-b99b-d5cfef7382f1:begin { provider: 'gemini -y -m gemini-2.5-pro' }  -->
Of course. Here is a step-by-step plan to develop the snake game:
@done

1.  **Project Setup:** I will initialize a new Rust project using Cargo and add the necessary dependencies: `ratatui` for the TUI, `crossterm` for terminal manipulation, and `rand` for random number generation.

@answer

2.  **Basic TUI Structure:** I'll create the foundation of the application, including initializing the terminal, setting up the main loop to draw frames, and handling basic input to exit the game.

3.  **Game State and Rendering:** I will define the data structures for the game, including the snake, the food, and the game area. Then, I'll implement the rendering logic to display these elements on the screen.

4.  **Snake Movement and User Control:** I'll implement the core game logic for the snake's movement and add event handling to allow the player to control the snake's direction using the keyboard.

5.  **Gameplay Mechanics:** I will add collision detection for the walls and the snake's own body, the logic for the snake to "eat" the food and grow longer, and the spawning of new food at random locations.

6.  **Scoring and Game Over:** Finally, I'll add a score display that updates as the snake eats food and a "Game Over" screen that appears when the game ends, with an option to restart.

Shall I start with the first step?
<!-- answer-87151488-88d3-4648-b99b-d5cfef7382f1:end {}  -->


