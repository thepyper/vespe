// src/screens/splash_screen.rs
use std::time::{Instant, Duration};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
    style::{Style, Color, Modifier},
    text::{Span, Spans},
};
use crate::app::App;

pub struct SplashScreen {
    start_time: Instant,
    animation_frame: usize,
    // Add ASCII art frames here later
}

impl SplashScreen {
    pub fn new() -> Self {
        SplashScreen {
            start_time: Instant::now(),
            animation_frame: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.start_time.elapsed() >= Duration::from_secs(5)
    }

    pub fn update_animation(&mut self) {
        // Logic to update animation_frame based on time or ticks
        // For now, a simple increment
        self.animation_frame = (self.animation_frame + 1) % 4; // Assuming 4 frames for now
    }

    pub fn get_current_frame(&self) -> usize {
        self.animation_frame
    }
}

// ASCII art for the splash screen
const SPLASH_ART: [&str; 4] = [
    r#"
  _   _      _             _         _   _              
 | | (_)    | |           | |       | | (_)             
 | |  _  ___| | ___  _ __ | | __ _  | |_ _  ___ ___  ___
 | | | |/ __| |/ _ \| '_ \| |/ _` | | __| |/ __/ _ \/ __|
 | | | | (__| | (_) | | | | | (_| | | |_| | (_|  __/\__ \
 |_| |_|\___|_|\___/|_| |_|_|\__,_|  \__|_|\___||___||___/
                                                         
                                                         
            "#,
    r#"
  _   _      _             _         _   _              
 | | (_)    | |           | |       | | (_)             
 | |  _  ___| | ___  _ __ | | __ _  | |_ _  ___ ___  ___
 | | | |/ __| |/ _ \| '_ \| |/ _` | | __| |/ __/ _ \/ __|
 | | | | (__| | (_) | | | | | (_| | | |_| | (_|  __/\__ \
 |_| |_|\___|_|\___/|_| |_|_|\__,_|  \__|_|\___||___||___/
                                                         
                                                         
            "#,
    r#"
  _   _      _             _         _   _              
 | | (_)    | |           | |       | | (_)             
 | |  _  ___| | ___  _ __ | | __ _  | |_ _  ___ ___  ___
 | | | |/ __| |/ _ \| '_ \| |/ _` | | __| |/ __/ _ \/ __|
 | | | | (__| | (_) | | | | | (_| | | |_| | (_|  __/\__ \
 |_| |_|\___|_|\___/|_| |_|_|\__,_|  \__|_|\___||___||___/
                                                         
                                                         
            "#,
    r#"
  _   _      _             _         _   _              
 | | (_)    | |           | |       | | (_)             
 | |  _  ___| | ___  _ __ | | __ _  | |_ _  ___ ___  ___
 | | | |/ __| |/ _ \| '_ \| |/ _` | | __| |/ __/ _ \/ __|
 | | | | (__| | (_) | | | | | (_| | | |_| | (_|  __/\__ \
 |_| |_|\___|_|\___/|_| |_|_|\__,_|  \__|_|\___||___||___/
                                                         
                                                         
            "#,
];

pub fn render<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let block = Block::default().borders(Borders::ALL).title("Tic-Crab-Toe");
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(f.size());

    let current_frame_index = app.splash_screen.get_current_frame();
    let splash_text = SPLASH_ART[current_frame_index];

    let paragraph = Paragraph::new(splash_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, chunks[0]);

    let loading_text = Spans::from(vec![
        Span::styled(
            "Loading...",
            Style::default().fg(Color::White).add_modifier(Modifier::ITALIC),
        ),
    ]);

    let loading_paragraph = Paragraph::new(loading_text)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(loading_paragraph, chunks[1]);
}

