use ratatui::{prelude::*, widgets::*};

pub fn render_agents_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Agents Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
}
