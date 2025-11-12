use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::game::GameState;

pub fn draw(frame: &mut Frame, game: &GameState) {
    let area = frame.size();
    let main_block = Block::new()
        .borders(Borders::ALL)
        .title("Snake")
        .title_alignment(Alignment::Center);
    frame.render_widget(main_block, area);

    let game_area = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);

    if game.game_over {
        let text = vec![
            Line::from("".bold()),
            Line::from("Game Over".bold()),
            Line::from("".bold()),
            Line::from(format!("Score: {}", game.score).bold()),
            Line::from("".bold()),
            Line::from("Press 'r' to restart".italic()),
            Line::from("Press 'q' to quit".italic()),
        ];
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::new().borders(Borders::NONE));
        frame.render_widget(paragraph, area);
        return;
    }

    // Draw snake
    for &(x, y) in &game.snake.body {
        if x < game_area.width + game_area.x && y < game_area.height + game_area.y {
            frame.render_widget(
                Paragraph::new("■").style(Style::default().fg(Color::Green)),
                Rect::new(x, y, 1, 1),
            );
        }
    }

    // Draw food
    frame.render_widget(
        Paragraph::new("•").style(Style::default().fg(Color::Red)),
        Rect::new(game.food.0, game.food.1, 1, 1),
    );

    // Draw score
    let score_text = format!("Score: {}", game.score);
    frame.render_widget(
        Paragraph::new(score_text).alignment(Alignment::Right),
        Rect::new(area.width - 12, 0, 10, 1),
    );
}
