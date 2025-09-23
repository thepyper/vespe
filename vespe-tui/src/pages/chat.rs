use ratatui::{prelude::*, widgets::*};

pub fn render_chat_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Chat Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
}
