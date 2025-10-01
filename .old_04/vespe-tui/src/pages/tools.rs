use ratatui::{prelude::*, widgets::*};

pub fn render_tools_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Tools Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
}
