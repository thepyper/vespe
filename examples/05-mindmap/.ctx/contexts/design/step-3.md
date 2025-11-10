**Detailed Implementation Plan for Step 3: Implement Splash Screen**

**Objective:** Create and integrate a splash screen with a "funny" ASCII art, a "blinking" or "shining" animation effect, and a 5-second timer before transitioning to the main menu.

**Affected Files:**
- `src/screens/mod.rs`
- `src/screens/splash_screen.rs` (new file)
- `src/app.rs`
- `src/ui.rs`
- `src/event.rs`

**Plan:**

1.  **Create `src/screens/splash_screen.rs`:**
    *   Define a `SplashScreen` struct with fields:
        *   `start_time: std::time::Instant`
        *   `animation_frame: usize`
        *   `ascii_art_frames: Vec<String>` (will contain different ASCII art states for animation)
    *   Implement a `new()` method for `SplashScreen` to initialize `start_time` to `Instant::now()`, `animation_frame` to 0, and populate `ascii_art_frames` with initial ASCII art designs.
    *   Implement an `update()` method for `SplashScreen` that takes an `AppEvent::Tick`. This method will:
        *   Increment `animation_frame` and cycle it based on the number of `ascii_art_frames`.
        *   Check if `Instant::now().duration_since(self.start_time)` is greater than or equal to `Duration::from_secs(5)`.
    *   Implement an `is_finished()` method for `SplashScreen` that returns `true` if the 5-second duration has passed.
    *   Implement a `render()` method for `SplashScreen` that takes a `ratatui::Frame` and renders the current `ascii_art_frames[self.animation_frame]` using `ratatui::widgets::Paragraph` centered on the screen.

2.  **Update `src/screens/mod.rs`:**
    *   Add `pub mod splash_screen;` to declare the new module.
    *   Import `SplashScreen` and add `SplashScreen(splash_screen::SplashScreen)` as a variant to the `CurrentScreen` enum.

3.  **Update `src/app.rs`:**
    *   Modify the `App` struct to include a `current_screen: CurrentScreen` field.
    *   Update the `App::new()` method to initialize `current_screen` to `CurrentScreen::SplashScreen(SplashScreen::new())`.
    *   Modify the `App::update()` method to handle `AppEvent::Tick`:
        *   If `self.current_screen` is `CurrentScreen::SplashScreen(splash_screen)`, call `splash_screen.update(event)`.
        *   If `splash_screen.is_finished()` is `true`, change `self.current_screen` to `CurrentScreen::MainMenu`. (The `MainMenu` struct and `CurrentScreen::MainMenu` variant will be implemented in a later step, for now, it can be a placeholder or a simple `println!` to indicate transition).

4.  **Update `src/ui.rs`:**
    *   Modify the `render_ui()` function to match on `app.current_screen`:
        *   If `app.current_screen` is `CurrentScreen::SplashScreen(splash_screen)`, call `splash_screen.render(frame)`.
        *   For other `CurrentScreen` variants, provide a placeholder rendering (e.g., a simple `Paragraph` with "Main Menu" text).

5.  **Refine `src/event.rs`:**
    *   Ensure the `AppEvent` enum has a `Tick` variant.
    *   The main loop in `main.rs` (from Step 2) should already be sending `AppEvent::Tick` periodically. Ensure the tick rate is appropriate for animation (e.g., 250ms).

**Pre-computation/Pre-analysis:**
- Design a simple ASCII art for "Tic-Crab-Toe" and create at least two variations for a basic blinking effect. For example, one with a character and one without, or two different characters.

**Verification Steps:**
1.  Run `cargo check` to ensure the code compiles without errors.
2.  Execute the application.
3.  Observe the splash screen displaying the ASCII art with a blinking/shining effect for 5 seconds.
4.  Verify that after 5 seconds, the application automatically transitions to a placeholder for the main menu (e.g., a screen displaying "Main Menu").

Please confirm if this detailed plan is acceptable before I proceed with the implementation.
